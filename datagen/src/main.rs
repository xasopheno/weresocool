use num_rational::Rational64;
use scop::Defs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use weresocool_ast::{NameSet, NormalForm, Normalize, Op, PointOp, Term};
use weresocool_shared::helpers::f32_to_rational;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGData {
    data: Vec<f32>,
}

fn main() {
    let data = get_data();
    let result = vec_normal_form_to_normal_form(data);
    dbg!(result);
}

fn vec_normal_form_to_normal_form(data: Vec<EEGData>) -> NormalForm {
    // 41,700 long and 4 minutes and 37 seconds. 277 sec
    let mut seqs: Vec<NormalForm> = data
        .iter()
        .map(|stream| vec_eeg_data_to_normal_form(stream))
        .collect();
    let overlay = Op::Overlay {
        operations: seqs.iter_mut().map(|nf| Term::Nf(nf.to_owned())).collect(),
    };
    let mut nf = NormalForm::init();
    overlay
        .apply_to_normal_form(&mut nf, &mut Defs::new())
        .expect("unable to normalize");
    nf
}

fn vec_eeg_data_to_normal_form(data: &EEGData) -> NormalForm {
    let point_ops: Vec<PointOp> = data
        .data
        .iter()
        .map(|value| eeg_datum_to_point_op(*value, 8))
        .collect();

    NormalForm {
        length_ratio: f32_to_rational(point_ops.len() as f32 * 0.02),
        operations: vec![point_ops],
    }
}

fn eeg_datum_to_point_op(datum: f32, idx: usize) -> PointOp {
    let mut nameset = NameSet::new();
    nameset.insert(format!("eeg{}", idx));
    let fa = f32_to_rational(datum * 100_000.0);
    PointOp {
        fm: Rational64::new(1, 1),
        fa,
        l: Rational64::new(1, 1),
        g: Rational64::new(1, 1),
        pm: Rational64::new(1, 1),
        pa: Rational64::new(1, 1),
        asr: weresocool_ast::ASR::Long,
        portamento: Rational64::new(1, 1),
        attack: Rational64::new(1, 1),
        decay: Rational64::new(1, 1),
        reverb: None,
        osc_type: weresocool_ast::OscType::Sine { pow: None },
        names: nameset,
    }
}

fn get_data() -> Vec<EEGData> {
    let file_path = "data/sample_audvis_filt-0-40_raw_chanel_EEG_008_array_0.csv";
    let file = File::open(file_path).expect("unable to read file");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(file);

    rdr.deserialize::<EEGData>().map(|t| t.unwrap()).collect()
}
