//! A DIAGNOSTIC, not a regression test. Prints the events a piece normalizes
//! to, per voice, so a question like "is there an extra note?" is answered by
//! reading the score the engine actually built rather than by inferring it
//! from a rendered waveform.
//!
//!     DUMP=/path/to/piece.socool cargo test --test dump_events -- --ignored --nocapture

use weresocool_ast::{NormalForm, Normalize as NormalizeOp};
use weresocool_parser::*;

#[test]
#[ignore = "diagnostic; set DUMP=<file.socool>"]
fn dump_events() {
    let path = std::env::var("DUMP").expect("set DUMP=<file.socool>");
    let vec_string = filename_to_vec_string(&path).unwrap();
    let mut parsed = parse_file(vec_string, None, None, Some(path.clone())).unwrap();
    let main = parsed.defs.ops.get("main").expect("no `main`").clone();

    let mut nf = NormalForm::init();
    main.apply_to_normal_form(&mut nf, &mut parsed.defs).unwrap();

    println!("\n{} voices, length_ratio {}", nf.operations.len(), nf.length_ratio);
    for (vi, voice) in nf.operations.iter().enumerate() {
        let total: num_rational::Rational64 =
            voice.iter().fold(num_rational::Rational64::new(0, 1), |a, p| a + p.l);
        println!("\nvoice {vi}: {} events, total length {}", voice.len(), total);
        let mut t = num_rational::Rational64::new(0, 1);
        for (i, p) in voice.iter().enumerate() {
            println!(
                "  {i:>3}  t={t:>8}  l={:>8}  fm={:>8}  fa={:>5}  g={:>6}  gate={:>5}  {}",
                p.l,
                p.fm,
                p.fa,
                p.g,
                p.gate,
                if p.is_silent() { "SILENT" } else { "" },
            );
            t += p.l;
        }
    }
}
