//! Cleromancy's divination authority as a wasm component (spike).
//!
//! The candidate arrives as one JSON object, `{"snapshot": ..., "facets": ...}`:
//! the kernel's `GraphSnapshot` of the complete candidate graph plus its facet
//! store, which the snapshot alone does not carry (the spike's findings). The
//! guest restores the graph and runs the same `validate_divination_graph` the
//! native resident runs. It imports `log` only: no clock, no store, no keys.

wit_bindgen::generate!({
    path: "../wit",
    world: "authority-guest",
});

use exports::spike::authority::domain_authority::{Claim, Guest};
use mere::kernel::graph::Graph;

struct Divination;

impl Guest for Divination {
    fn describe() -> Claim {
        Claim {
            domain: "divination".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            graph_format: "mere.graph-snapshot-and-facets/json".into(),
        }
    }

    fn validate(candidate: Vec<u8>) -> Result<(), String> {
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&candidate).map_err(|error| format!("candidate: {error}"))?;
        let snapshot = serde_json::from_value(candidate["snapshot"].take())
            .map_err(|error| format!("candidate snapshot: {error}"))?;
        let facets = serde_json::from_value(candidate["facets"].take())
            .map_err(|error| format!("candidate facets: {error}"))?;
        let mut graph =
            Graph::try_from_snapshot(&snapshot).map_err(|error| format!("candidate: {error}"))?;
        graph
            .install_facets(facets)
            .map_err(|error| format!("candidate facets: {error:?}"))?;
        cleromancy::host::validate_divination_graph(&graph).map_err(|error| error.to_string())
    }

    fn admit(_before: Vec<u8>, _after: Vec<u8>) -> Result<(), String> {
        // Divination registers no before/after check (`admission: None`).
        Ok(())
    }
}

export!(Divination);
