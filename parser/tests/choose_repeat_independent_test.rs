use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};
use std::collections::HashSet;

#[test]
fn test_choose_repeat_should_make_independent_choices() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[Tm 1/1, Tm 5/4, Tm 3/2] | Repeat 4
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing Choose | Repeat (SHOULD make independent choices) ===\n");
    println!("Number of voices: {}", nf.operations.len());

    assert_eq!(nf.operations.len(), 1, "Expected 1 voice");

    let voice = &nf.operations[0];
    println!("Events: {}", voice.len());

    // Should have 4 events from Repeat 4
    assert_eq!(voice.len(), 4, "Expected exactly 4 events from Repeat 4");

    // Get the frequency multipliers
    let fms: Vec<_> = voice.iter().map(|p| p.fm).collect();
    println!("Frequency multipliers: {:?}", fms);

    // Count unique values
    let unique: HashSet<_> = fms.iter().cloned().collect();
    println!("Unique FMs: {} out of {}", unique.len(), fms.len());

    // THE KEY TEST: We should have MORE THAN ONE unique value
    // With 4 repetitions choosing from 3 options, we expect variety
    println!("\nCurrent behavior check:");
    if unique.len() == 1 {
        println!("❌ FAILS: All 4 events have the SAME value: {:?}", fms[0]);
        println!("   This means Choose was evaluated ONCE and duplicated 4 times.");
        println!("   Expected: Each Repeat iteration makes a fresh random choice.");
    } else {
        println!("✅ PASSES: Found {} different values", unique.len());
        println!("   Choose is being evaluated independently for each Repeat iteration.");
    }

    // For now, we EXPECT this to fail with the bug
    // After the fix, this should pass
    assert!(
        unique.len() > 1,
        "\n\n🐛 BUG CONFIRMED: Choose | Repeat 4 produced only ONE unique value {:?}.\n\
         All 4 events are identical, meaning Choose was evaluated once and the result was duplicated.\n\
         Expected: 4 independent random choices.\n\n",
        fms[0]
    );
}

#[test]
fn test_nested_choose_repeat() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Overlay [
        {1/1, 2, 1, 1},
        {1/1, 0, 1, -1}
    ]
    | Choose [Fm 1, Fm 9/8, Fm 5/4, Fm 1/2, Fm 15/16]
    | Repeat 8
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing Overlay | Choose | Repeat ===\n");
    println!("Number of voices: {}", nf.operations.len());

    // Overlay creates 2 voices, Repeat 8 should create 8 segments
    assert_eq!(nf.operations.len(), 2, "Expected 2 voices from Overlay");

    let voice_0 = &nf.operations[0];
    println!("Voice 0 events: {}", voice_0.len());
    println!("Voice 1 events: {}", nf.operations[1].len());

    // Each voice should have 8 events (one per Repeat iteration)
    assert_eq!(voice_0.len(), 8, "Expected 8 events in voice 0");

    // Get the frequency multipliers
    let fms: Vec<_> = voice_0.iter().map(|p| p.fm).collect();
    println!("Voice 0 FMs: {:?}", fms);

    // Count unique values
    let unique: HashSet<_> = fms.iter().cloned().collect();
    println!("Unique FMs: {} out of {}", unique.len(), fms.len());

    println!("\nCurrent behavior check:");
    if unique.len() == 1 {
        println!("❌ FAILS: All 8 events have the SAME FM: {:?}", fms[0]);
        println!("   This confirms the bug: Choose evaluated once, result duplicated.");
    } else {
        println!("✅ PASSES: Found {} different FM values across 8 iterations", unique.len());
        println!("   Each Repeat iteration is making an independent Choice.");
    }

    assert!(
        unique.len() > 1,
        "\n\n🐛 BUG CONFIRMED: Overlay | Choose | Repeat 8 produced only ONE unique FM.\n\
         Expected: 8 independent random choices.\n\n"
    );
}

#[test]
fn test_choose_without_repeat_still_works() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[Tm 1/1, Tm 5/4, Tm 3/2]
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing simple Choose (sanity check) ===\n");
    println!("Number of voices: {}", nf.operations.len());

    assert_eq!(nf.operations.len(), 1, "Expected 1 voice");

    let voice = &nf.operations[0];
    println!("Events: {}", voice.len());

    // Should have 1 event
    assert_eq!(voice.len(), 1, "Expected 1 event from simple Choose");

    let fm = voice[0].fm;
    println!("Chosen FM: {:?}", fm);

    // Should be one of the three options
    let valid = [
        num_rational::Ratio::from_integer(1),
        num_rational::Ratio::new(5, 4),
        num_rational::Ratio::new(3, 2),
    ];
    assert!(
        valid.contains(&fm),
        "FM should be one of the valid options, got {:?}",
        fm
    );

    println!("✅ Simple Choose works correctly");
}
