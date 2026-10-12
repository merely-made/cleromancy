// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unmodified OFL faces; register native metadata, preserving styles/weights.
use cambium_genet_winit_host::HostFont;
use std::collections::BTreeMap;
use tabard::typography::{FontFamily, FontRole};
pub(super) fn bundled_fonts() -> Vec<HostFont> {
    [
        include_bytes!("../../../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf").as_slice(),
        include_bytes!("../../../assets/fonts/ibm-plex-mono/IBMPlexMono-Bold.ttf").as_slice(),
        include_bytes!("../../../assets/fonts/ibm-plex-mono/IBMPlexMono-Italic.ttf").as_slice(),
        include_bytes!("../../../assets/fonts/ibm-plex-mono/IBMPlexMono-BoldItalic.ttf").as_slice(),
        include_bytes!("../../../assets/fonts/source-serif-4/SourceSerif4.ttf").as_slice(),
        include_bytes!("../../../assets/fonts/source-serif-4/SourceSerif4-Italic.ttf").as_slice(),
    ]
    .into_iter()
    .map(|bytes| HostFont {
        family: None,
        bytes: bytes.to_vec(),
    })
    .collect()
}
pub(super) fn defaults() -> BTreeMap<FontRole, FontFamily> {
    FontRole::ALL
        .into_iter()
        .map(|role| {
            (
                role,
                FontFamily::new(match role {
                    FontRole::Reading | FontRole::Heading => "serif",
                    FontRole::Code => "monospace",
                    _ => "sans-serif",
                })
                .unwrap(),
            )
        })
        .collect()
}
