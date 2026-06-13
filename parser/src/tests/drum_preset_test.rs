//! Parse tests for the drum preset syntax:
//!   Kick | Kick 808 | Kick 808 { tune: 1/2 } | Kick { hump: 2 }
//! Unknown preset names must be parse errors that list the available names.

use crate::parser::{language_to_vec_string, parse_file};
use num_rational::Rational64;
use weresocool_ast::{OscType, Term};

/// Parse `main = { <body> }` and return the osc types of the first voice's
/// point ops (definitions are stored in normal form).
fn parse_main(body: &str) -> Result<Vec<OscType>, String> {
    let src = format!("{{ f: 220, l: 1, g: 1, p: 0 }}\n\nmain = {{ {} }}\n", body);
    let parsed = parse_file(language_to_vec_string(&src), None, None, None)
        .map_err(|e| format!("{:?}", e))?;
    match parsed.defs.ops.get("main").expect("main not defined") {
        Term::Nf(nf) => Ok(nf.operations[0]
            .iter()
            .map(|p| p.osc_type.clone())
            .collect()),
        other => Err(format!("expected Term::Nf, got {:?}", other)),
    }
}

fn single_osc(body: &str) -> OscType {
    let oscs = parse_main(body).unwrap();
    assert_eq!(oscs.len(), 1, "expected a single op for `{}`", body);
    oscs.into_iter().next().unwrap()
}

#[test]
fn bare_kick_still_parses_with_no_params() {
    match single_osc("Kick") {
        OscType::Kick { params } => assert!(params.is_none()),
        other => panic!("expected Kick, got {:?}", other),
    }
}

#[test]
fn kick_with_numeric_preset() {
    match single_osc("Kick 808") {
        OscType::Kick { params } => {
            let p = params.expect("preset should produce params");
            assert_eq!(p.preset.as_deref(), Some("808"));
            assert!(p.tune.is_none());
        }
        other => panic!("expected Kick, got {:?}", other),
    }
}

#[test]
fn kick_with_preset_and_overrides() {
    match single_osc("Kick 808 { tune: 1/2, hump: 2 }") {
        OscType::Kick { params } => {
            let p = params.expect("params");
            assert_eq!(p.preset.as_deref(), Some("808"));
            assert_eq!(p.tune, Some(Rational64::new(1, 2)));
            assert_eq!(p.hump, Some(Rational64::new(2, 1)));
        }
        other => panic!("expected Kick, got {:?}", other),
    }
}

#[test]
fn kick_with_params_only_keeps_no_preset() {
    match single_osc("Kick { hump: 2 }") {
        OscType::Kick { params } => {
            let p = params.expect("params");
            assert!(p.preset.is_none());
            assert_eq!(p.hump, Some(Rational64::new(2, 1)));
        }
        other => panic!("expected Kick, got {:?}", other),
    }
}

#[test]
fn snare_and_hat_named_presets() {
    match single_osc("Snare trap") {
        OscType::Snare { params } => {
            assert_eq!(params.unwrap().preset.as_deref(), Some("trap"));
        }
        other => panic!("expected Snare, got {:?}", other),
    }
    match single_osc("HiHat dust") {
        OscType::HiHat { open, params } => {
            assert!(!open);
            assert_eq!(params.unwrap().preset.as_deref(), Some("dust"));
        }
        other => panic!("expected HiHat, got {:?}", other),
    }
    match single_osc("OpenHat 909") {
        OscType::HiHat { open, params } => {
            assert!(open);
            assert_eq!(params.unwrap().preset.as_deref(), Some("909"));
        }
        other => panic!("expected HiHat, got {:?}", other),
    }
}

#[test]
fn preset_inside_sequence() {
    // The preset token must not interfere with list separators.
    let oscs = parse_main("Seq [Kick 808, Snare wsc, HiHat trap]").unwrap();
    assert_eq!(oscs.len(), 3);
    assert!(matches!(&oscs[0], OscType::Kick { params: Some(p) } if p.preset.as_deref() == Some("808")));
    assert!(matches!(&oscs[1], OscType::Snare { params: Some(p) } if p.preset.as_deref() == Some("wsc")));
    assert!(matches!(&oscs[2], OscType::HiHat { open: false, params: Some(p) } if p.preset.as_deref() == Some("trap")));
}

#[test]
fn unknown_preset_is_an_error_listing_available() {
    let err = parse_main("Kick zzz").unwrap_err();
    assert!(
        err.contains("zzz"),
        "error should mention the bad preset: {}",
        err
    );
}

#[test]
fn clap_and_rimshot_presets_parse() {
    match single_osc("Clap 808") {
        OscType::Clap { params } => {
            assert_eq!(params.unwrap().preset.as_deref(), Some("808"));
        }
        other => panic!("expected Clap, got {:?}", other),
    }
    match single_osc("Rimshot acoustic") {
        OscType::Rimshot { params } => {
            assert_eq!(params.unwrap().preset.as_deref(), Some("acoustic"));
        }
        other => panic!("expected Rimshot, got {:?}", other),
    }
    match single_osc("Clap") {
        OscType::Clap { params } => assert!(params.is_none()),
        other => panic!("expected Clap, got {:?}", other),
    }
}
