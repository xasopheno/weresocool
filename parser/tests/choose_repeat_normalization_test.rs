use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};
use num_rational::Ratio;

#[test]
fn test_choose_repeat_produces_different_values() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
  Choose[Tm 1/1, Tm 5/4, Tm 3/2, Tm 2/1] | Repeat 4
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("Normalized form:");
    println!("  Length ratio: {}", nf.length_ratio);
    println!("  Number of operations: {}", nf.operations.len());

    // With 4 repeats, we should have 4 events in sequence
    // Each one should be 1 beat long (l: 1.0)
    assert_eq!(nf.operations.len(), 1, "Expected 1 voice");

    let voice = &nf.operations[0];
    println!("  Events in voice: {}", voice.len());

    // Each repeat creates events, so we should have multiple events
    assert!(voice.len() >= 4, "Expected at least 4 events from 4 repeats");

    // Let's check that the frequency multipliers are different
    // (This verifies that Choose is making different choices)
    let mut fms: Vec<_> = voice.iter().map(|point_op| point_op.fm).collect();
    fms.sort();
    fms.dedup();

    println!("  Unique frequency multipliers: {:?}", fms);

    // With 4 choices and deterministic randomness, we should get some variety
    // (not necessarily all 4 different, but definitely not all the same)
    assert!(
        fms.len() > 1,
        "Expected different frequency multipliers from Choose, but all were the same: {:?}",
        fms
    );
}

#[test]
fn test_simple_repeat_without_choose() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
  Tm 3/2 | Repeat 3
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("Simple repeat normalized form:");
    println!("  Length ratio: {}", nf.length_ratio);
    println!("  Number of voices: {}", nf.operations.len());

    // Should have 3 beats total (3 repeats × 1 beat each)
    assert_eq!(nf.length_ratio, Ratio::from_integer(3));

    assert_eq!(nf.operations.len(), 1, "Expected 1 voice");

    let voice = &nf.operations[0];
    println!("  Events in voice: {}", voice.len());

    // All events should have the same fm (3/2)
    let fm = Ratio::new(3, 2);
    for (i, point_op) in voice.iter().enumerate() {
        assert_eq!(
            point_op.fm, fm,
            "Event {} has fm={:?}, expected {:?}",
            i, point_op.fm, fm
        );
    }
}
