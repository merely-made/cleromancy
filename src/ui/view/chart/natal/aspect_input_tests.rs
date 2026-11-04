// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
fn ui() -> ConsultationUi {
    let consultation =
        crate::Consultation::new(crate::CleromancyHost::empty(muniment::MemoryBackend::new()));
    ConsultationUi::new(consultation.catalog().unwrap())
}
#[test]
fn explicit_orbs_parse_exactly_and_refuse_invalid_input() {
    let mut ui = ui();
    assert!(!ui.natal_aspects_enabled);
    assert_eq!(
        aspect_rules(&ui).unwrap(),
        crate::natal_aspects::AspectRules::default()
    );
    ui.natal_aspect_orbs[0] = TextInput::new("0.001");
    assert_eq!(aspect_rules(&ui).unwrap().conjunction_millidegrees, 1);
    for invalid in ["-1", "180.001", "NaN", "1.0001", ""] {
        ui.natal_aspect_orbs[0] = TextInput::new(invalid);
        assert!(aspect_rules(&ui).is_err(), "{invalid}");
    }
    ui.natal_aspect_orbs[0] = TextInput::new("0");
    assert_eq!(aspect_rules(&ui).unwrap().conjunction_millidegrees, 0);
}
