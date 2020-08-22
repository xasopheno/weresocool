use walkdir::WalkDir;
// use weresocool::generation::nn::{get_min_max_for_path, CSVData, CSVOp};
// use num_rational::Rational64;
use weresocool::generation::{RenderReturn, RenderType};
use weresocool::interpretable::{InputType::Filename, Interpretable};
use weresocool_ast::PointOp;
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
}

impl DataOp {
    fn from_point_op(op: PointOp) -> Self {
        Self {
            fm: r_to_f64(op.fm),
            fa: r_to_f64(op.fa),
            g: r_to_f64(op.g),
            l: r_to_f64(op.l),
            pm: r_to_f64(op.pm),
            pa: r_to_f64(op.pa),
        }
    }
}

fn main() -> Result<(), Error> {
    let op = PointOp::init();
    let data_op = DataOp::from_point_op(op);
    dbg!(data_op);

    let mut max_state = DataOp {
        fm: 0.0,
        fa: 0.0,
        g: 0.0,
        l: 0.0,
        pm: 0.0,
        pa: 0.0,
    };
    let mut min_state = DataOp {
        fm: 0.0,
        fa: 0.0,
        g: 0.0,
        l: 0.0,
        pm: 0.0,
        pa: 0.0,
    };

    // let mut max_seq_length = 0;

    for entry in WalkDir::new("./songs/training_data")
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

            let data_ops: Vec<Vec<DataOp>> = nf
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
                            };
                            min_state = DataOp {
                                fm: min_state.fm.min(data_op.fm),
                                fa: min_state.fa.min(data_op.fa),
                                pm: min_state.pm.min(data_op.pm),
                                pa: min_state.pa.min(data_op.pa),
                                g: min_state.g.min(data_op.g),
                                l: min_state.l.min(data_op.l),
                            };

                            data_op
                        })
                        .collect()
                })
                .collect();
            dbg!(data_ops);

            // let (data_op, data_op, n_voices) = get_min_max_for_path(f_name.to_string());

            // max_seq_length = max_seq_length.max(n_voices);

            // println!("{:#?}", n_voices)
        }
    }
    println!("MAX {:#?}\nMIN {:#?}\n", max_state, min_state);

    Ok(())
}
