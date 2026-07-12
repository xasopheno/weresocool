//! Smoke tests for the `Perform("name")` op + recordings registry.

use weresocool_core::generation::{RenderReturn, RenderType};
use weresocool_core::interpretable::{make_with_recordings, InputType};

const SRC: &str = "{ f: 311.127, l: 1, g: 1, p: 0 }\nmain = { Fm 1 | Perform(\"sax\") }\n";

#[test]
fn perform_miss_does_not_error_and_is_pending() {
    let result = make_with_recordings(
        &InputType::Language(SRC),
        RenderType::NfBasisAndTable,
        None,
        weresocool_ast::RecordingRegistry::new(),
    )
    .expect("Perform miss should render, not error");

    match result {
        RenderReturn::NfBasisAndTable(_, _, defs) => {
            assert!(
                defs.pending_performs.contains("sax"),
                "unresolved Perform name should be collected in pending_performs"
            );
        }
        _ => panic!("unexpected render return"),
    }
}

#[test]
fn perform_hit_resolves_recording() {
    // Build a recording NF by rendering a tiny composition, seed it under
    // "sax", and confirm Perform("sax") is no longer pending.
    let rec_src = "{ f: 311.127, l: 1, g: 1, p: 0 }\nmain = { Overlay [Fm 1, Fm 5/4] }\n";
    let rec_nf = match make_with_recordings(
        &InputType::Language(rec_src),
        RenderType::NfBasisAndTable,
        None,
        weresocool_ast::RecordingRegistry::new(),
    )
    .expect("recording source should render")
    {
        RenderReturn::NfBasisAndTable(nf, _, _) => nf,
        _ => panic!("unexpected render return"),
    };

    let mut recordings = weresocool_ast::RecordingRegistry::new();
    recordings.insert("sax".to_string(), rec_nf);

    let result = make_with_recordings(
        &InputType::Language(SRC),
        RenderType::NfBasisAndTable,
        None,
        recordings,
    )
    .expect("Perform hit should render");

    match result {
        RenderReturn::NfBasisAndTable(_, _, defs) => {
            assert!(
                !defs.pending_performs.contains("sax"),
                "resolved Perform name must not be pending"
            );
        }
        _ => panic!("unexpected render return"),
    }
}
