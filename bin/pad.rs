use walkdir::WalkDir;
// use num_rational::Rational64;
use weresocool::generation::{RenderReturn, RenderType};
use weresocool::interpretable::{InputType::Filename, Interpretable};
use weresocool_ast::{OscType, PointOp};
use weresocool_error::Error;
use weresocool_shared::helpers::r_to_f64;

#[derive(Debug, Clone, Copy)]
struct DataOp {
    fm: f64,
    fa: f64,
    g: f64,
    l: f64,
    pm: f64,
    pa: f64,
    osc_type: f64,
}

impl DataOp {
    fn from_point_op(op: PointOp) -> Self {
        let osc_type = match op.osc_type {
            OscType::Sine => 0.0,
            _ => 1.0,
        };
        Self {
            fm: r_to_f64(op.fm),
            fa: r_to_f64(op.fa),
            g: r_to_f64(op.g),
            l: r_to_f64(op.l),
            pm: r_to_f64(op.pm),
            pa: r_to_f64(op.pa),
            osc_type,
        }
    }
}

fn normalize(x: f64, min_x: f64, max_x: f64) -> f64 {
    (x - min_x) / (max_x - min_x)
}

#[test]
fn test_normalize() {
    let result = normalize(8.0, 0.0, 10.0);
    let expected = 0.8;
    assert_eq!(result, expected);
}

// let op = PointOp::init();
// let data_op = DataOp::from_point_op(op);
// dbg!(data_op);

fn main() -> Result<(), Error> {
    let mut max_state = DataOp {
        fm: 0.0,
        fa: 0.0,
        g: 0.0,
        l: 0.0,
        pm: 0.0,
        pa: 0.0,
        osc_type: 1.0,
    };
    let mut min_state = DataOp {
        fm: 0.0,
        fa: 0.0,
        g: 0.0,
        l: 0.0,
        pm: 0.0,
        pa: 0.0,
        osc_type: 0.0,
    };

    for entry in WalkDir::new("./application/extraResources/demo/")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.path().to_string_lossy();
        if f_name.ends_with(".socool") {
            println!("{:?}", f_name);
            let render_return = Filename(&f_name).make(RenderType::NfBasisAndTable, None)?;
            let (nf, _, _) = match render_return {
                RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
                _ => panic!("huh"),
            };

            let _data_ops: Vec<Vec<DataOp>> = nf
                .operations
                .iter()
                .map(|voice| {
                    voice
                        .iter()
                        .map(|op| {
                            let data_op = DataOp::from_point_op(op.to_owned());
                            max_state = DataOp {
                                fm: max_state.fm.max(data_op.fm),
                                fa: max_state.fa.max(data_op.fa),
                                pm: max_state.pm.max(data_op.pm),
                                pa: max_state.pa.max(data_op.pa),
                                g: max_state.g.max(data_op.g),
                                l: max_state.l.max(data_op.l),
                                ..max_state
                            };
                            min_state = DataOp {
                                fm: min_state.fm.min(data_op.fm),
                                fa: min_state.fa.min(data_op.fa),
                                pm: min_state.pm.min(data_op.pm),
                                pa: min_state.pa.min(data_op.pa),
                                g: min_state.g.min(data_op.g),
                                l: min_state.l.min(data_op.l),
                                ..min_state
                            };

                            data_op
                        })
                        .collect()
                })
                .collect();
        }
    }
    println!("MAX {:#?}\nMIN {:#?}\n", max_state, min_state);

    Ok(())
}
