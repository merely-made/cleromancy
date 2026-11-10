// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's divination authority as a wasm component (cycle breaks P3).
//!
//! The candidate arrives as one JSON object, `{"snapshot": ..., "facets": ...}`
//! (`mere.graph-snapshot-and-facets/json`): the kernel's snapshot of the complete candidate
//! graph plus its facet store, which the snapshot alone does not carry. The guest restores
//! the graph and runs the same `validate_divination_graph` the native resident ran. It
//! imports `log` only: it cannot read a key, a vault, a clock or a store.
//!
//! Divination registers no before/after check, so the claim says `admission: false` and the
//! host never calls `admit`.

wit_bindgen::generate!({
    path: "wit",
    world: "authority-guest",
});

use exports::mere::script::domain_authority::{Claim, Guest};
use ::kernel::graph::{Graph, NodeFacetStore};
use serde::Deserialize;

/// The grammar version of the divination domain this component validates (the registration's
/// `version`, shown in status).
const VERSION: &str = "v1";

/// The declared format of the candidate bytes.
pub const GRAPH_FORMAT: &str = "mere.graph-snapshot-and-facets/json";

/// The snapshot's type is the kernel's (`GraphSnapshot`, not exported by name); it is fixed
/// by `Graph::try_from_snapshot` below.
#[derive(Deserialize)]
struct Candidate<S> {
    snapshot: S,
    facets: NodeFacetStore,
}

/// Restore the candidate graph from the declared format.
pub fn restore(candidate: &[u8]) -> Result<Graph, String> {
    let Candidate { snapshot, facets } =
        serde_json::from_slice::<Candidate<_>>(candidate).map_err(|error| format!("candidate: {error}"))?;
    let mut graph =
        Graph::try_from_snapshot(&snapshot).map_err(|error| format!("candidate: {error}"))?;
    graph
        .install_facets(facets)
        .map_err(|error| format!("candidate facets: {error:?}"))?;
    Ok(graph)
}

struct Divination;

impl Guest for Divination {
    fn describe() -> Claim {
        Claim {
            domain: "divination".into(),
            version: VERSION.into(),
            graph_format: GRAPH_FORMAT.into(),
            admission: false,
        }
    }

    fn validate(candidate: Vec<u8>) -> Result<(), String> {
        let graph = restore(&candidate)?;
        cleromancy::host::validate_divination_graph(&graph).map_err(|error| error.to_string())
    }

    fn admit(_before: Vec<u8>, _after: Vec<u8>) -> Result<(), String> {
        // Never called: the claim says there is no before/after check.
        Ok(())
    }
}

export!(Divination);
