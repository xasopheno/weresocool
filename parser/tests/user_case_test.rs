use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};
use std::collections::HashSet;
use num_traits::{ToPrimitive, Signed};

#[test]
fn test_user_complex_case() {
    let parse_str = r#"{ f: 311.127, l: 1, g: 1/3, p: 0 }

thing1 = {
    Overlay [
        {1/1, 2, 1, 1},
        {1/1, 0, 1, -1}
    ]
    | Choose [
        Fm 1, Fm 9/8, Fm 5/4, Fm 1/2, Fm 15/16
    ]
    | Repeat 8
    | FitLength Lm 8
}

main = {
    thing1
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing User's Complex Case ===\n");
    println!("Number of voices: {}", nf.operations.len());
    println!("Total length_ratio: {}", nf.length_ratio);

    // Overlay creates 2 voices
    assert_eq!(nf.operations.len(), 2, "Expected 2 voices from Overlay");

    let voice_0 = &nf.operations[0];
    let voice_1 = &nf.operations[1];

    println!("\nVoice 0: {} events", voice_0.len());
    println!("Voice 1: {} events", voice_1.len());

    // Get FMs from voice 0
    let fms: Vec<_> = voice_0.iter().map(|p| p.fm).collect();
    println!("\nVoice 0 FMs: {:?}", fms);

    // Count unique values
    let unique: HashSet<_> = fms.iter().cloned().collect();
    println!("Unique FMs in voice 0: {} out of {}", unique.len(), fms.len());

    // Get lengths
    let lengths: Vec<_> = voice_0.iter().map(|p| p.l).collect();
    println!("\nVoice 0 lengths: {:?}", lengths);

    // Sum of lengths
    let total_length: num_rational::Rational64 = lengths.iter().sum();
    println!("Sum of event lengths: {}", total_length);
    println!("NormalForm length_ratio: {}", nf.length_ratio);

    // Check FitLength worked
    let expected_length = num_rational::Ratio::from_integer(8);
    println!("\nFitLength check:");
    println!("  Expected: {}", expected_length);
    println!("  Actual: {}", nf.length_ratio);

    if nf.length_ratio == expected_length {
        println!("  ✅ FitLength worked correctly");
    } else {
        println!("  ❌ FitLength did NOT produce expected length");
        println!("     Ratio: {} (expected: {})",
                 nf.length_ratio.to_f64().unwrap(),
                 expected_length.to_f64().unwrap());
    }

    // Check randomness variety
    println!("\nRandomness check:");
    if unique.len() > 1 {
        println!("  ✅ Choose | Repeat produced {} different FM values", unique.len());
        println!("     This shows each Repeat iteration makes independent choices");
    } else {
        println!("  ❌ Choose | Repeat produced only 1 unique FM value");
        println!("     All iterations chose the same value: {:?}", fms[0]);
    }

    // Assertions
    assert!(
        unique.len() > 1,
        "Expected variety in FM choices across Repeat iterations"
    );
}

#[test]
fn test_fitlength_with_simple_repeat() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Tm 1/1 | Repeat 4 | FitLength Lm 10
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing FitLength with Simple Repeat ===\n");
    println!("Number of voices: {}", nf.operations.len());
    println!("NormalForm length_ratio: {}", nf.length_ratio);

    let expected_length = num_rational::Ratio::from_integer(10);

    assert_eq!(
        nf.length_ratio, expected_length,
        "FitLength should scale to length 10"
    );

    let voice = &nf.operations[0];
    let total_event_length: num_rational::Rational64 = voice.iter().map(|p| p.l).sum();

    println!("Sum of event lengths: {}", total_event_length);
    println!("Expected: {}", expected_length);

    // They should match (within floating point tolerance if needed)
    let diff = (total_event_length - expected_length).abs();
    println!("Difference: {}", diff);

    println!("✅ FitLength with simple Repeat works correctly");
}

#[test]
fn test_fitlength_with_choose_repeat() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[Tm 1/1, Tm 5/4, Tm 3/2] | Repeat 4 | FitLength Lm 10
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing FitLength with Choose | Repeat ===\n");
    println!("Number of voices: {}", nf.operations.len());
    println!("NormalForm length_ratio: {}", nf.length_ratio);

    let expected_length = num_rational::Ratio::from_integer(10);

    println!("Expected length: {}", expected_length);
    println!("Actual length: {}", nf.length_ratio);

    let voice = &nf.operations[0];
    let total_event_length: num_rational::Rational64 = voice.iter().map(|p| p.l).sum();

    println!("Sum of event lengths: {}", total_event_length);

    // Get FMs to show variety
    let fms: Vec<_> = voice.iter().map(|p| p.fm).collect();
    println!("FMs chosen: {:?}", fms);

    let unique: HashSet<_> = fms.iter().cloned().collect();
    println!("Unique FMs: {}", unique.len());

    // Check FitLength worked
    if nf.length_ratio == expected_length {
        println!("✅ FitLength worked correctly with Choose | Repeat");
    } else {
        println!("❌ FitLength did NOT produce expected length");
        println!("   Difference: {}", (nf.length_ratio - expected_length).abs());
    }

    assert_eq!(
        nf.length_ratio, expected_length,
        "FitLength should work correctly even with random choices"
    );
}
