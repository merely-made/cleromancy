// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Headed, semantic consultation receipts.
//!
//! `CLEROMANCY_SCENARIO` names a genet-probe scenario. Its companion phase is
//! deliberately explicit: `first` authors a consultation and reflection;
//! `reopen` selects the same durable session from a fresh process. Captures are
//! composed from the presented Genet scene, not from an occluded desktop.

use std::path::{Path, PathBuf};

use image::ImageEncoder;
use netrender::ExternalTexturePlacement;
use serde::Serialize;

use genet_winit_host::SurfaceHost;

pub use genet_probe::{Outcome, Scenario};

/// Which half of the close/reopen receipt a process is executing.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase {
    First,
    Reopen,
}

impl Phase {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "first" => Ok(Self::First),
            "reopen" => Ok(Self::Reopen),
            _ => Err(format!(
                "CLEROMANCY_SCENARIO_PHASE must be first or reopen, got {value:?}"
            )),
        }
    }
}

/// A loaded scenario plus its receipt destination and required phase.
pub(crate) struct Run {
    pub(crate) scenario: Scenario,
    pub(crate) dir: PathBuf,
    pub(crate) phase: Phase,
}

/// Load a self-drive scenario, or return `None` for a normal interactive run.
pub(crate) fn load() -> Option<Run> {
    let path = PathBuf::from(std::env::var_os("CLEROMANCY_SCENARIO")?);
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read scenario {path:?}: {error}"));
    let scenario =
        Scenario::parse(&body).unwrap_or_else(|error| panic!("parse scenario {path:?}: {error}"));
    let phase = match std::env::var("CLEROMANCY_SCENARIO_PHASE") {
        Ok(value) => {
            Phase::parse(&value).unwrap_or_else(|error| panic!("load Cleromancy scenario: {error}"))
        }
        Err(_) => panic!("load Cleromancy scenario: CLEROMANCY_SCENARIO_PHASE is required"),
    };
    let dir = std::env::var_os("CLEROMANCY_CAPTURE_DIR")
        .map(PathBuf::from)
        .or_else(|| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|error| panic!("create scenario receipt directory {dir:?}: {error}"));
    Some(Run {
        scenario,
        dir,
        phase,
    })
}

/// The durable identities the relaunch harness compares without scraping text.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReportedIds {
    pub(crate) session_id: String,
    pub(crate) reading_id: String,
    pub(crate) reflection_id: String,
}

/// App state observed when a headed scenario completes.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Observation {
    pub(crate) status: String,
    pub(crate) catalog_ready: bool,
    pub(crate) sessions: usize,
    pub(crate) readings: usize,
    pub(crate) reflections: usize,
    pub(crate) ids: Option<ReportedIds>,
}

#[derive(Serialize)]
struct Receipt {
    schema: &'static str,
    phase: Phase,
    ok: bool,
    status: String,
    catalog_ready: bool,
    sessions: usize,
    readings: usize,
    reflections: usize,
    ids: Option<ReportedIds>,
    log: Vec<String>,
}

/// Write both a concise greppable result and a typed JSON observation.
pub(crate) fn write_done(
    dir: &Path,
    phase: Phase,
    outcome: &Outcome,
    observation: Observation,
    capture_error: Option<&str>,
) {
    let mut ok = outcome.ok && observation.catalog_ready && observation.ids.is_some();
    let mut log = outcome.log.clone();
    if !observation.catalog_ready {
        log.push("FAIL: local catalog did not become ready".to_string());
    }
    if observation.ids.is_none() {
        log.push(
            "FAIL: scenario completed without durable session, reading, and reflection ids"
                .to_string(),
        );
    }
    if let Some(error) = capture_error {
        ok = false;
        log.push(format!("FAIL: scenario capture: {error}"));
    }

    let mut done = format!("RESULT {}\n", if ok { "ok" } else { "fail" });
    for line in &log {
        done.push_str(line);
        done.push('\n');
    }
    std::fs::write(dir.join("scenario.done"), done)
        .unwrap_or_else(|error| panic!("write scenario.done in {dir:?}: {error}"));

    let receipt = Receipt {
        schema: "cleromancy.headed-scenario/v1",
        phase,
        ok,
        status: observation.status,
        catalog_ready: observation.catalog_ready,
        sessions: observation.sessions,
        readings: observation.readings,
        reflections: observation.reflections,
        ids: observation.ids,
        log,
    };
    let json = serde_json::to_vec_pretty(&receipt).expect("serialize headed scenario receipt");
    std::fs::write(dir.join("receipt.json"), json)
        .unwrap_or_else(|error| panic!("write receipt.json in {dir:?}: {error}"));
}

/// Capture the just-presented composed scene to a PNG for a pixel receipt.
pub(crate) fn capture_frame(
    host: &SurfaceHost,
    view: &wgpu::TextureView,
    width: u32,
    height: u32,
    path: &Path,
) -> bool {
    let target = host.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("cleromancy scenario capture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    host.renderer().compose_external_texture(
        view,
        &target_view,
        wgpu::TextureFormat::Rgba8Unorm,
        width,
        height,
        ExternalTexturePlacement::new([0.0, 0.0, width as f32, height as f32]),
    );
    let rgba = read_texture_rgba(host.device(), host.queue(), &target, width, height);
    if rgba.is_empty() {
        return false;
    }
    let Ok(file) = std::fs::File::create(path) else {
        return false;
    };
    image::codecs::png::PngEncoder::new(file)
        .write_image(&rgba, width, height, image::ExtendedColorType::Rgba8)
        .is_ok()
}

fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let row_bytes = width * 4;
    let padded = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("cleromancy capture readback"),
        size: padded as u64 * height as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("cleromancy capture readback"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    if device.poll(wgpu::PollType::wait_indefinitely()).is_err() || !matches!(rx.recv(), Ok(Ok(())))
    {
        return Vec::new();
    }
    let Ok(mapped) = slice.get_mapped_range() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity((row_bytes * height) as usize);
    for row in 0..height as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + row_bytes as usize]);
    }
    out
}
