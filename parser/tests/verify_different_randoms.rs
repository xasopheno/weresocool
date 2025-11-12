use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};
use num_rational::Ratio;

#[test]
fn verify_thing1_instances_produce_different_sequences() {
    let parse_str = r#"{ f: 311.127, l: 1, g: 1/3, p: 0 }

thing1 = {
    Overlay [
        {1/1, 2, 1, 1},
        {1/1, 0, 1, -1}
    ]
    | Choose [
        Fm 1, Fm 9/8, Fm 5/4
    ] | Repeat 8
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

    println!("\n=== Testing Reused Definition Randomness ===\n");
    println!("Number of voices: {}", nf.operations.len());

    // thing1 contains Overlay[{...}, {...}], so it produces 2 voices
    // When we have Overlay[thing1|Fm3, thing1|Fm2], we get 4 voices total:
    // - Voices 0-1: from first thing1 (with Fm 3 applied)
    // - Voices 2-3: from second thing1 (with Fm 2 applied)
    assert_eq!(nf.operations.len(), 4, "Expected 4 voices: 2 from each thing1 instance");

    // Compare the first voice from first thing1 with first voice from second thing1
    let voice_0_0 = &nf.operations[0];  // First thing1, first internal voice
    let voice_1_0 = &nf.operations[2];  // Second thing1, first internal voice

    println!("\nFirst thing1 instance (| Fm 3), first voice:");
    println!("  Total events: {}", voice_0_0.len());
    println!("  First 10 event FMs:");
    for (i, point_op) in voice_0_0.iter().take(10).enumerate() {
        println!("    Event {}: fm = {}", i, point_op.fm);
    }

    println!("\nSecond thing1 instance (| Fm 2), first voice:");
    println!("  Total events: {}", voice_1_0.len());
    println!("  First 10 event FMs:");
    for (i, point_op) in voice_1_0.iter().take(10).enumerate() {
        println!("    Event {}: fm = {}", i, point_op.fm);
    }

    // Collect the frequency patterns
    let fms_0: Vec<_> = voice_0_0.iter().take(16).map(|p| p.fm).collect();
    let fms_1: Vec<_> = voice_1_0.iter().take(16).map(|p| p.fm).collect();

    println!("\n=== Comparison ===");
    println!("Voice 0 pattern: {:?}", fms_0);
    println!("Voice 1 pattern: {:?}", fms_1);

    // THE CRITICAL ASSERTION: The two instances should produce DIFFERENT random sequences
    assert_ne!(
        fms_0, fms_1,
        "\n\n❌ FAILED: Both thing1 instances produced IDENTICAL random sequences!\n\
         This means the randomness context is not being properly differentiated.\n\n"
    );

    println!("\n✅ SUCCESS: The two thing1 instances produced DIFFERENT random sequences!");
    println!("   This proves that reused definitions get independent random contexts.\n");
}

#[test]
fn verify_choose_produces_variety() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[Tm 1/1, Tm 5/4, Tm 3/2, Tm 2/1] | Repeat 8
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing Choose Variety ===\n");

    assert_eq!(nf.operations.len(), 1, "Expected 1 voice");

    let voice = &nf.operations[0];
    println!("Events: {}", voice.len());

    let fms: Vec<_> = voice.iter().map(|p| p.fm).collect();
    println!("Frequency multipliers: {:?}", fms);

    // Get unique values
    let unique: std::collections::HashSet<_> = fms.iter().cloned().collect();
    println!("Unique FMs: {} out of {}", unique.len(), fms.len());

    // With 8 repeats and 4 choices, we should see some variety
    assert!(
        unique.len() > 1,
        "Choose should produce variety across 8 repetitions, but all values were the same: {:?}",
        fms[0]
    );

    println!("✅ Choose produced {} different values across {} repetitions\n", unique.len(), fms.len());

    // List the actual chosen sequence with their ratios
    println!("Sequence of choices:");
    for (i, fm) in fms.iter().enumerate() {
        let ratio_str = if *fm == Ratio::from_integer(1) {
            "1/1".to_string()
        } else if *fm == Ratio::new(5, 4) {
            "5/4".to_string()
        } else if *fm == Ratio::new(3, 2) {
            "3/2".to_string()
        } else if *fm == Ratio::from_integer(2) {
            "2/1".to_string()
        } else {
            format!("{}", fm)
        };
        println!("  Repeat {}: Tm {}", i + 1, ratio_str);
    }
}
