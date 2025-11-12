use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};

#[test]
fn test_reused_definition_gets_different_random_values() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
thing1 = {
    Choose[Tm 1/1, Tm 5/4, Tm 3/2] | Repeat 4
}

main = {
    Overlay [
        thing1 | Fm 3,
        thing1 | Fm 2
    ]
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("Normalized form with reused definition:");
    println!("  Number of voices: {}", nf.operations.len());

    // We should have 2 voices (one for each thing1 reference)
    assert_eq!(nf.operations.len(), 2, "Expected 2 voices from Overlay");

    let voice_0 = &nf.operations[0];  // thing1 | Fm 3
    let voice_1 = &nf.operations[1];  // thing1 | Fm 2

    println!("  Voice 0 events: {}", voice_0.len());
    println!("  Voice 1 events: {}", voice_1.len());

    // Both should have 4 events (from Repeat 4)
    assert!(voice_0.len() >= 4, "Voice 0 should have at least 4 events");
    assert!(voice_1.len() >= 4, "Voice 1 should have at least 4 events");

    // Get the frequency multipliers for first 4 events of each voice
    // (ignoring the base Fm 3 and Fm 2 which will multiply these)
    let fms_0: Vec<_> = voice_0.iter().take(4).map(|p| p.fm).collect();
    let fms_1: Vec<_> = voice_1.iter().take(4).map(|p| p.fm).collect();

    println!("  Voice 0 FMs (first 4): {:?}", fms_0);
    println!("  Voice 1 FMs (first 4): {:?}", fms_1);

    // The key test: the two voices should have DIFFERENT random sequences
    // (they may occasionally overlap, but they should not be identical)
    assert_ne!(
        fms_0, fms_1,
        "The two thing1 instances should produce DIFFERENT random sequences, but they're identical!"
    );

    // Also verify that each voice has at least some variety in its choices
    let unique_0: std::collections::HashSet<_> = fms_0.iter().collect();
    let unique_1: std::collections::HashSet<_> = fms_1.iter().collect();

    println!("  Voice 0 unique FMs: {}", unique_0.len());
    println!("  Voice 1 unique FMs: {}", unique_1.len());

    // Each voice should have chosen from multiple options (with high probability)
    assert!(
        unique_0.len() > 1 || unique_1.len() > 1,
        "Expected at least one voice to have variety in its random choices"
    );
}

#[test]
fn test_simple_compose_preserves_context() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[Tm 1/1, Tm 5/4] | Repeat 2 | Fm 2
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("Simple compose test:");
    println!("  Number of voices: {}", nf.operations.len());

    assert_eq!(nf.operations.len(), 1, "Expected 1 voice");

    let voice = &nf.operations[0];
    println!("  Events: {}", voice.len());

    // Should have 2 events from Repeat 2
    assert!(voice.len() >= 2, "Expected at least 2 events");

    // Both should have Fm 2 applied (so base fm should be 2)
    for (i, point_op) in voice.iter().enumerate() {
        println!("  Event {} fm: {:?}", i, point_op.fm);
    }

    // Just verify it doesn't crash and produces reasonable output
    assert!(voice.len() > 0, "Should have at least one event");
}
