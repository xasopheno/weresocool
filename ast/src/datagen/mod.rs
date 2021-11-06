use crate::{NameSet, NormalForm, Normalize, Op, OscType, PointOp, Term, ASR};
use num_rational::Rational64;
use scop::Defs;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use weresocool_shared::helpers::f32_to_rational;

mod test;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGData {
    data: Vec<f32>,
}

pub fn csv_to_normalform(filename: &str, scale: f32) -> NormalForm {
    // let file_path = "data/sample_audvis_filt-0-40_raw_chanel_EEG_008_array_0.csv";
    let data = get_data(filename.into());
    vec_eeg_data_to_normal_form(data, scale)
}

fn vec_eeg_data_to_normal_form(data: Vec<EEGData>, scale: f32) -> NormalForm {
    let mut nfs: Vec<NormalForm> = data
        .iter()
        .map(|stream| eeg_data_to_normal_form(stream, scale))
        .collect();

    let overlay = Op::Overlay {
        operations: nfs.iter_mut().map(|nf| Term::Nf(nf.to_owned())).collect(),
    };

    let mut nf = NormalForm::init();
    overlay
        .apply_to_normal_form(&mut nf, &mut Defs::new())
        .expect("unable to normalize");
    nf
}

fn eeg_data_to_normal_form(data: &EEGData, scale: f32) -> NormalForm {
    let mut length_ratio = Rational64::new(0, 1);
    let point_ops: Vec<PointOp> = data
        .data
        .iter()
        .map(|value| {
            let op = eeg_datum_to_point_op(*value, 1, scale);
            length_ratio += op.l;
            op
        })
        .collect();

    NormalForm {
        length_ratio,
        operations: vec![point_ops],
    }
}

fn eeg_datum_to_point_op(datum: f32, idx: usize, scale: f32) -> PointOp {
    let mut nameset = NameSet::new();
    nameset.insert(format!("eeg_{}", idx));
    let fa = f32_to_rational(datum * scale);
    PointOp {
        fm: Rational64::new(1, 1),
        fa,
        l: Rational64::new(2, 100),
        g: Rational64::new(1, 1),
        pm: Rational64::new(1, 1),
        pa: Rational64::new(0, 1),
        asr: ASR::Long,
        portamento: Rational64::new(1, 1),
        attack: Rational64::new(1, 1),
        decay: Rational64::new(1, 1),
        reverb: None,
        osc_type: OscType::None,
        names: nameset,
    }
}

fn get_data(filename: String) -> Vec<EEGData> {
    let path = Path::new(&filename);
    let file = File::open(path).expect("unable to read file");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(file);

    rdr.deserialize::<EEGData>().map(|t| t.unwrap()).collect()
}
