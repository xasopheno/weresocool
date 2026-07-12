use weresocool_parser::parser::*;
use weresocool_ast::{Defs, NormalForm, Normalize};
use weresocool_ast::rand_ctx::RandCtx;
use num_rational::Ratio;
use num_traits::Signed;

#[test]
fn test_fitlength_with_variable_length_choose() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[
        Tm 1/1,
        Tm 5/4 | Lm 2,
        Tm 3/2 | Lm 1/2
    ]
    | Repeat 8
    | FitLength Lm 10
}"#;

    let mut defs: Defs = Default::default();

    let _init = socool::SoCoolParser::new().parse(&mut defs, parse_str).unwrap();

    let main = defs.ops.get("main").unwrap().clone();

    // Normalize to NormalForm
    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut defs).unwrap();

    println!("\n=== Testing FitLength with Variable-Length Choose Options ===\n");
    println!("Choose options have lengths: 1, 2, 0.5");
    println!("Repeat 8 times, FitLength to 10");
    println!();

    let expected_length = Ratio::from_integer(10);
    println!("Expected final length: {}", expected_length);
    println!("Actual NormalForm length_ratio: {}", nf.length_ratio);

    // Check if lengths match
    if nf.length_ratio == expected_length {
        println!("✅ FitLength worked correctly!");
    } else {
        println!("❌ FitLength FAILED!");
        println!("   Difference: {}", (nf.length_ratio - expected_length));
    }

    assert_eq!(
        nf.length_ratio, expected_length,
        "FitLength should scale to exactly 10, regardless of random choices"
    );

    // Verify the actual event lengths also sum correctly
    let voice = &nf.operations[0];
    let total_event_length: num_rational::Rational64 = voice.iter().map(|p| p.l).sum();
    println!("\nSum of individual event lengths: {}", total_event_length);

    // Should be very close (might have tiny rounding differences)
    let diff = (total_event_length - expected_length).abs();
    println!("Difference from expected: {}", diff);

    assert!(
        diff < Ratio::new(1, 1000),
        "Sum of event lengths should match target length"
    );
}

#[test]
fn test_user_complex_case_fitlength() {
    let parse_str = r#"{ f: 311.127, l: 1, g: 1/3, p: 0 }

thing1 = {
    Overlay [
        {1/1, 2, 1, 1},
        {1/1, 0, 1, -1}
    ]
    | Choose [
        Fm 1,
        Fm 9/8,
        Fm 5/4,
        Fm 1/2,
        Fm 15/16,
        Fm 3/2 | Overlay [Fm 1/2, Fm 3/2, Fm 1],
        Fm 2/3 | Lm 2,
        Seq [Fm 3/2, Fm 5/6] | Repeat 4 | Lm 1/4
    ]
    | Repeat 8
    | FitLength Lm 8
}

main = {
    Overlay [
        thing1 | Fm 2,
        thing1 | Fm 3/2,
        thing1 | Fm 1,
        thing1 | Fm 1/2
    ]
    | Lm 1/2
    | Repeat 3
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

    // After Lm 1/2 and Repeat 3:
    // thing1 has FitLength Lm 8, so length = 8
    // After Lm 1/2: length = 8 * 1/2 = 4
    // After Repeat 3: length = 4 * 3 = 12
    let expected_length = Ratio::from_integer(12);

    println!("Expected length: {}", expected_length);
    println!("Actual length: {}", nf.length_ratio);

    assert_eq!(
        nf.length_ratio, expected_length,
        "Final length should be 12"
    );

    // Check that all voices have the same total length
    println!("\nPer-voice lengths:");
    for (i, voice) in nf.operations.iter().enumerate() {
        let voice_length: num_rational::Rational64 = voice.iter().map(|p| p.l).sum();
        println!("  Voice {}: {} events, total length: {}", i, voice.len(), voice_length);
    }

    // All voices should end at the same time
    let voice_lengths: Vec<_> = nf.operations.iter()
        .map(|voice| voice.iter().map(|p| p.l).sum::<num_rational::Rational64>())
        .collect();

    let first_length = voice_lengths[0];
    let all_same = voice_lengths.iter().all(|&len| len == first_length);

    if all_same {
        println!("\n✅ All voices end at the same time: {}", first_length);
    } else {
        println!("\n❌ Voices end at DIFFERENT times!");
        for (i, len) in voice_lengths.iter().enumerate() {
            println!("   Voice {}: {}", i, len);
        }
    }

    assert!(
        all_same,
        "All voices should have the same total length after FitLength"
    );

    println!("\n✅ FitLength working correctly with complex Choose options!");
}

#[test]
fn test_fitlength_preserves_determinism() {
    let parse_str = r#"{ f: 200, l: 1.0, g: 1.0, p: 0.0 }
main = {
    Choose[Tm 1/1, Tm 5/4 | Lm 2, Tm 3/2 | Lm 1/2]
    | Repeat 8
    | FitLength Lm 10
}"#;

    // Create both defs with the same seed to ensure determinism
    let seed = 0x1234_5678_9ABC_DEF0;
    let mut defs1 = Defs {
        ops: Default::default(),
        colors: weresocool_ast::color::ColorMap::new(),
        wgsl: weresocool_ast::wgsl::WgslMap::new(),
        rand_ctx: RandCtx::from_u128(seed),
        spans: weresocool_ast::SpanMap::new(),
        recordings: Default::default(),
        pending_performs: Default::default(),
    };
    let mut defs2 = Defs {
        ops: Default::default(),
        colors: weresocool_ast::color::ColorMap::new(),
        wgsl: weresocool_ast::wgsl::WgslMap::new(),
        rand_ctx: RandCtx::from_u128(seed),
        spans: weresocool_ast::SpanMap::new(),
        recordings: Default::default(),
        pending_performs: Default::default(),
    };

    let _init1 = socool::SoCoolParser::new().parse(&mut defs1, parse_str).unwrap();
    let _init2 = socool::SoCoolParser::new().parse(&mut defs2, parse_str).unwrap();

    let main1 = defs1.ops.get("main").unwrap().clone();
    let main2 = defs2.ops.get("main").unwrap().clone();

    let mut nf1 = NormalForm::init();
    let mut nf2 = NormalForm::init();

    main1.apply_to_normal_form(&mut nf1, &mut defs1).unwrap();
    main2.apply_to_normal_form(&mut nf2, &mut defs2).unwrap();

    println!("\n=== Testing Determinism ===\n");
    println!("Run 1 length: {}", nf1.length_ratio);
    println!("Run 2 length: {}", nf2.length_ratio);

    // Both should produce the same final length
    assert_eq!(
        nf1.length_ratio, nf2.length_ratio,
        "FitLength should be deterministic"
    );

    // Both should have the same number of events
    assert_eq!(
        nf1.operations[0].len(),
        nf2.operations[0].len(),
        "Should produce same number of events"
    );

    // Events should be identical
    for (i, (p1, p2)) in nf1.operations[0].iter().zip(nf2.operations[0].iter()).enumerate() {
        assert_eq!(
            p1.fm, p2.fm,
            "Event {} fm should match",
            i
        );
        assert_eq!(
            p1.l, p2.l,
            "Event {} length should match",
            i
        );
    }

    println!("✅ FitLength is deterministic!");
}
