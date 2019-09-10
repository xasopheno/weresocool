use error::Error;
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use socool_ast::PointOp;
use weresocool::generation::{filename_to_render, RenderReturn, RenderType};
use weresocool::write::filename_from_string;
use csv::Writer;
use std::path::Path;

pub fn r_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvOp {
    pub fm: f64,
    pub fa: f64,
    pub pm: f64,
    pub pa: f64,
    pub g: f64,
    pub l: f64,
}

impl From<&PointOp> for CsvOp {
    fn from(p_op: &PointOp) -> CsvOp {
        CsvOp {
            fm: r_to_f64(p_op.fm),
            fa: r_to_f64(p_op.fa),
            pm: r_to_f64(p_op.pm),
            pa: r_to_f64(p_op.pa),
            g: r_to_f64(p_op.g),
            l: r_to_f64(p_op.l),
        }
    }
}

pub fn write_voice_to_csv(ops: Vec<CsvOp>, filename: &str, n: usize) -> Result<(), Error> {
    let filename = filename_from_string(filename);

    let filename = &format!("../data/{:0>4}_{}{}", n.to_string(), filename,  ".socool.csv".to_string());

    //dbg!(filename);

    let path = Path::new(filename);
    let mut writer = Writer::from_path(&path)?;
    for op in ops {
        writer
            .serialize(op)
            .ok()
            .expect("CSV writer error");
    }
    
    Ok(())
}


fn main() -> Result<(), Error> {
    println!("Hello Scratch Pad");
    let filename = "songs/transcriptions/bach_prelude_2.socool";

    match filename_to_render(filename, RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, _basis, _table) => {
            dbg!(nf.operations.len());
            let result: Vec<Vec<CsvOp>> = nf
                .operations
                .iter()
                .map(|voice| voice.iter().map(|op| CsvOp::from(op)).collect())
                .collect();

            dbg!(result.len());
            for (voice_n, voice) in result.iter().enumerate() {
                write_voice_to_csv(voice.to_vec(), &filename, voice_n);
            };
        },
        _ => panic!(),
    };

    Ok(())
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
