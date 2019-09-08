use weresocool::generation::{filename_to_render, RenderReturn, RenderType};
use error::Error;
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use socool_ast::PointOp;

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
            fa: r_to_f64(p_op.fm),
            pm: r_to_f64(p_op.fm),
            pa: r_to_f64(p_op.fm),
            g: r_to_f64(p_op.fm),
            l: r_to_f64(p_op.fm),
        }
    }
}

fn main() -> Result<(), Error> {
    println!("Hello Scratch Pad");
    let filename = "songs/template.socool";
   
    match filename_to_render(filename, RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, _basis, _table) => {
            let result: Vec<Vec<CsvOp>> = nf.operations.iter().map(|voice| {
                voice.iter().map(|op| CsvOp::from(op)).collect()
            }).collect();
                
            dbg!(result);
            //for (i, voice) in nf.operations.iter().enumerate() {
                //dbg!(i);
                //for (j, op) in voice.iter().enumerate() {
                    //dbg!(j, op);

                //}
            //}
        }
        _ => panic!()
    };
            
    Ok(())
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}
