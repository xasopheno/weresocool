use crate::{NameSet, NormalForm, Normalize, Op, OscType, PointOp, Term, ASR};
use num_rational::Rational64;
use scop::Defs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use weresocool_shared::helpers::f32_to_rational;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGData {
    data: Vec<f32>,
}

fn csv_to_normalform(filename: &str, scale: f32) {
    let file_path = "data/sample_audvis_filt-0-40_raw_chanel_EEG_008_array_0.csv";
    let data = get_data(file_path.into());
    let result = vec_eeg_data_to_normal_form(data, 100_000.0);
}

fn vec_eeg_data_to_normal_form(data: Vec<EEGData>, scale: f32) -> NormalForm {
    // 41,700 long and 4 minutes and 37 seconds. 277 sec
    let mut seqs: Vec<NormalForm> = data
        .iter()
        .map(|stream| eeg_data_to_normal_form(stream, scale))
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

fn get_data(file_path: String) -> Vec<EEGData> {
    let file = File::open(file_path).expect("unable to read file");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(file);

    rdr.deserialize::<EEGData>().map(|t| t.unwrap()).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_eeg_datum_to_point_op() {
        let mut names = NameSet::new();
        names.insert(format!("eeg_{}", 1));
        let result = eeg_datum_to_point_op(0.01, 1, 200.0);
        let expected = PointOp {
            fa: Rational64::new(2, 1),
            l: Rational64::new(1, 50),
            names,
            ..PointOp::default()
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eeg_datum_to_normal_form() {
        let eeg_data = EEGData {
            data: vec![0.01, 0.02, 0.03],
        };
        let mut names = NameSet::new();
        names.insert(format!("eeg_{}", 1));
        let result = eeg_data_to_normal_form(&eeg_data, 100.0);
        let expected = NormalForm {
            operations: vec![vec![
                PointOp {
                    fa: Rational64::new(1, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(2, 1),
                    l: Rational64::new(1, 50),
                    names: names.clone(),
                    ..PointOp::default()
                },
                PointOp {
                    fa: Rational64::new(3, 1),
                    l: Rational64::new(1, 50),
                    names,
                    ..PointOp::default()
                },
            ]],
            length_ratio: Rational64::new(3, 50),
        };
        assert_eq!(result, expected);
    }
}
