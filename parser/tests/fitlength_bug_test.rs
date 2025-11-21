// Test demonstrating the FitLength bug with Choose containing different-length options
use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};
use num_traits::{ToPrimitive, Signed};

#[test]
fn test_fitlength_with_different_length_options() {
    // This test demonstrates a bug where FitLength calculates the wrong scaling factor
    // when Choose contains options with DIFFERENT lengths.
    
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose [
        Fm 1,
        { 2/3, 2, 1, 0 },
        { 3/2, 1/2, 1, 0 }
    ]
    | Repeat 8
    | FitLength Lm 8
}"#;

    let mut defs: Defs = Default::default();
    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();
    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing FitLength Bug with Different-Length Choose Options ===\n");
    println!("Choose options:");
    println!("  - Fm 1                   (length: 1)");
    println!("  - {{ Fm 2/3, 2, 1, 0 }}    (length: 2)");
    println!("  - {{ Fm 3/2, 1/2, 1, 0 }}  (length: 0.5)");
    println!("\nExpected final length: 8");
    println!("Actual length_ratio: {}", nf.length_ratio);
    println!("Actual length_ratio (float): {}", nf.length_ratio.to_f64().unwrap());

    let voice = &nf.operations[0];
    let total_event_length: num_rational::Rational64 = voice.iter().map(|p| p.l).sum();
    println!("Sum of event lengths: {}", total_event_length);
    println!("Sum of event lengths (float): {}", total_event_length.to_f64().unwrap());

    // Show the individual event lengths to understand what happened
    let lengths: Vec<_> = voice.iter().map(|p| p.l).collect();
    println!("\nIndividual event lengths: {:?}", lengths);
    
    let expected_length = num_rational::Ratio::from_integer(8);
    
    // Calculate how far off we are
    let diff = (nf.length_ratio - expected_length).abs();
    let diff_percent = (diff.to_f64().unwrap() / 8.0) * 100.0;
    
    println!("\nDifference from expected: {}", diff);
    println!("Difference percentage: {:.2}%", diff_percent);

    if nf.length_ratio != expected_length {
        println!("\n❌ BUG CONFIRMED: FitLength produced wrong length!");
        println!("   This happens because get_length_ratio() made different");
        println!("   random choices than the actual normalization did.");
    } else {
        println!("\n✅ No bug detected (got lucky with random choices)");
    }

    // This assertion SHOULD pass but might fail due to the bug
    assert_eq!(
        nf.length_ratio, expected_length,
        "FitLength should produce length 8, but got {} (diff: {})",
        nf.length_ratio, diff
    );
}

#[test]
fn test_fitlength_deterministic_failure() {
    // This test uses a large number of repetitions to make the bug more likely
    // to show up.
    
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose [
        { 1, 1, 1, 0 },
        { 1, 4, 1, 0 }
    ]
    | Repeat 20
    | FitLength Lm 20
}"#;

    let mut defs: Defs = Default::default();
    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();
    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing FitLength with Extreme Length Difference ===\n");
    println!("Choose options:");
    println!("  - {{ Fm 1, 1, 1, 0 }}    (length: 1)");
    println!("  - {{ Fm 1, 4, 1, 0 }}    (length: 4)");
    println!("\nExpected final length: 20");
    println!("Actual length_ratio: {}", nf.length_ratio);
    
    let expected_length = num_rational::Ratio::from_integer(20);
    let diff = (nf.length_ratio - expected_length).abs();
    
    println!("Difference: {} ({}%)", diff, (diff.to_f64().unwrap() / 20.0) * 100.0);

    if diff > num_rational::Ratio::new(1, 10) {
        println!("\n❌ SIGNIFICANT BUG: Difference is > 5%");
    }

    assert_eq!(
        nf.length_ratio, expected_length,
        "FitLength failed with large length difference"
    );
}
