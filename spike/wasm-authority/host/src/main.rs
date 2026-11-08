//! The spike's native harness: loads the authority component under wasmtime and
//! runs it over the corpus written by `tests/authority_corpus.rs`, comparing the
//! verdict with the native validator's, and measuring per-call time and fuel.
//!
//! Usage: authority-host COMPONENT.wasm CORPUS_DIR

use std::path::PathBuf;
use std::time::{Duration, Instant};

use mere::kernel::graph::{CapturedDelta, replay_captured_deltas};
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, ResourceLimiter, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    path: "../wit",
    world: "authority-guest",
});

/// The most guest memory a call may hold (the host's cap, not a measurement).
const MEMORY_CAP: usize = 1 << 30;
/// Fuel per call unless `AUTHORITY_FUEL` says otherwise. The guest is cut off
/// when it spends it.
const FUEL: u64 = 400_000_000_000;

fn fuel() -> u64 {
    std::env::var("AUTHORITY_FUEL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(FUEL)
}

struct State {
    wasi: WasiCtx,
    table: ResourceTable,
    peak_memory: usize,
    logs: Vec<String>,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl ResourceLimiter for State {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        self.peak_memory = self.peak_memory.max(desired);
        Ok(desired <= MEMORY_CAP)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(true)
    }
}

impl spike::authority::log::Host for State {
    fn log(&mut self, message: String) {
        self.logs.push(message);
    }
}

/// The candidate the resident would hand a guest: the batch replayed into the
/// complete graph, as its snapshot and its facet store.
fn candidate_of(deltas: &serde_json::Value) -> Vec<u8> {
    let deltas: Vec<CapturedDelta> = serde_json::from_value(deltas.clone()).unwrap();
    let graph = replay_captured_deltas(deltas);
    serde_json::to_vec(&serde_json::json!({
        "snapshot": graph.to_snapshot_at(0),
        "facets": graph.facets(),
    }))
    .unwrap()
}

/// What the native resident does with the same bytes the guest receives.
fn native(bytes: &[u8]) -> Result<(), String> {
    let mut candidate: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| format!("candidate: {error}"))?;
    let snapshot = serde_json::from_value(candidate["snapshot"].take())
        .map_err(|error| format!("candidate snapshot: {error}"))?;
    let facets = serde_json::from_value(candidate["facets"].take())
        .map_err(|error| format!("candidate facets: {error}"))?;
    let mut graph =
        mere::kernel::graph::Graph::try_from_snapshot(&snapshot).map_err(|error| format!("candidate: {error}"))?;
    graph
        .install_facets(facets)
        .map_err(|error| format!("candidate facets: {error:?}"))?;
    cleromancy::host::validate_divination_graph(&graph).map_err(|error| error.to_string())
}

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn main() -> wasmtime::Result<()> {
    let mut args = std::env::args().skip(1);
    let component_path = PathBuf::from(args.next().expect("COMPONENT.wasm"));
    let corpus = PathBuf::from(args.next().expect("CORPUS_DIR"));
    let bytes = std::fs::read(&component_path)?;
    println!("component: {} bytes ({:.2} MiB; cap 16 MiB)", bytes.len(), bytes.len() as f64 / 1_048_576.0);

    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;
    let started = Instant::now();
    let component = Component::from_binary(&engine, &bytes)?;
    println!("compile (cranelift, first load): {:?}", started.elapsed());

    let mut linker: Linker<State> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    spike::authority::log::add_to_linker::<_, HasSelf<State>>(&mut linker, |state| state)?;
    let mut store = Store::new(
        &engine,
        State {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            peak_memory: 0,
            logs: Vec::new(),
        },
    );
    store.limiter(|state| state);
    store.set_fuel(fuel())?;
    let started = Instant::now();
    let guest = AuthorityGuest::instantiate(&mut store, &component, &linker)?;
    println!("instantiate: {:?}", started.elapsed());
    let claim = guest.spike_authority_domain_authority().call_describe(&mut store)?;
    println!(
        "describe: domain={} version={} format={}",
        claim.domain, claim.version, claim.graph_format
    );

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&corpus)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort();

    let (mut cases, mut agree, mut error_text_equal, mut disagreed) = (0, 0, 0, Vec::new());
    let (mut ok_cases, mut refused_cases, mut traps) = (0, 0, 0);
    let mut wasm_times = Vec::new();
    let mut native_times = Vec::new();
    let mut fuels = Vec::new();
    let mut by_size: Vec<(usize, Duration, Duration)> = Vec::new();
    for path in &entries {
        let record: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        let name = record["name"].as_str().unwrap().to_string();
        let candidate = candidate_of(&record["deltas"]);
        let corpus_ok = record["native_ok"].as_bool().unwrap();
        let corpus_error = record["native_error"].as_str().map(str::to_string);

        let started = Instant::now();
        let here = native(&candidate);
        let native_time = started.elapsed();
        assert_eq!(here.is_ok(), corpus_ok, "{name}: the corpus verdict reproduces natively");

        store.set_fuel(fuel())?;
        let before = store.get_fuel()?;
        let started = Instant::now();
        let outcome = guest
            .spike_authority_domain_authority()
            .call_validate(&mut store, &candidate);
        let wasm_time = started.elapsed();
        let spent = before - store.get_fuel()?;
        let verdict: Result<(), String> = match outcome {
            Ok(result) => result,
            Err(trap) => {
                traps += 1;
                println!("{name}: TRAP {trap:#}");
                Err(format!("trap: {trap}"))
            },
        };
        cases += 1;
        if verdict.is_ok() { ok_cases += 1 } else { refused_cases += 1 }
        if verdict.is_ok() == corpus_ok {
            agree += 1;
            if verdict.as_ref().err() == corpus_error.as_ref() {
                error_text_equal += 1;
            }
        } else {
            disagreed.push(name);
        }
        wasm_times.push(wasm_time);
        native_times.push(native_time);
        fuels.push(spent);
        by_size.push((candidate.len(), native_time, wasm_time));
    }

    println!("corpus: {cases} cases ({ok_cases} admitted, {refused_cases} refused by the guest), {traps} traps");
    println!("verdict-equal: {agree}/{cases}; identical error text: {error_text_equal}/{cases}");
    if !disagreed.is_empty() {
        println!("DISAGREED: {disagreed:?}");
    }
    wasm_times.sort();
    native_times.sort();
    fuels.sort();
    for (label, times) in [("native", &native_times), ("wasm", &wasm_times)] {
        let total: Duration = times.iter().sum();
        println!(
            "{label}: per call min {:?}  p50 {:?}  p90 {:?}  max {:?}  mean {:?}",
            times[0],
            percentile(times, 0.5),
            percentile(times, 0.9),
            times[times.len() - 1],
            total / times.len() as u32
        );
    }
    println!(
        "fuel per call: p50 {}  max {}",
        fuels[fuels.len() / 2],
        fuels[fuels.len() - 1]
    );
    println!(
        "guest memory peak requested: {:.1} MiB; logs: {}",
        store.data().peak_memory as f64 / 1_048_576.0,
        store.data().logs.len()
    );
    by_size.sort();
    let largest = by_size.last().unwrap();
    let smallest = by_size.first().unwrap();
    println!(
        "candidate size {} .. {} bytes; at the largest: native {:?}, wasm {:?}; at the smallest: native {:?}, wasm {:?}",
        smallest.0, largest.0, largest.1, largest.2, smallest.1, smallest.2
    );
    if !disagreed.is_empty() || traps > 0 {
        std::process::exit(1);
    }
    Ok(())
}
