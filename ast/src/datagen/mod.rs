use crate::{NameSet, NormalForm, Normalize, Op, OscType, PointOp, Term, ASR};
use num_rational::{Ratio, Rational64};
use scop::Defs;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{fs::File, path::Path};
use weresocool_error::Error;
use weresocool_ring_buffer::RingBuffer;
use weresocool_shared::helpers::r_to_f32;
mod test;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvData {
    data: Vec<f32>,
}

pub fn csv_to_normalform(filename: &str, scale: Option<Rational64>) -> Result<NormalForm, Error> {
    let data = get_data(filename.into())?;
    let path = Path::new(&filename);
    Ok(vec_eeg_data_to_normal_form(
        data,
        if let Some(s) = scale {
            r_to_f32(s)
        } else {
            1.0
        },
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .as_str(),
    ))
}

fn vec_eeg_data_to_normal_form(data: Vec<CsvData>, scale: f32, filename: &str) -> NormalForm {
    let mut nfs: Vec<NormalForm> = data
        .iter()
        .map(|stream| eeg_data_to_normal_form(stream, scale, filename))
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

fn eeg_data_to_normal_form(data: &CsvData, scale: f32, filename: &str) -> NormalForm {
    let mut length_ratio = Rational64::new(0, 1);

    let mut buffer = RingBuffer::<f32>::new(50);

    let point_ops: Vec<PointOp> = data
        .data
        .iter()
        .map(|value| {
            let op = eeg_datum_to_point_op(*value, Some(&mut buffer), scale, filename);
            length_ratio += op.l;
            op
        })
        .collect();

    NormalForm {
        length_ratio,
        operations: vec![point_ops],
    }
}

pub fn f32_to_rational(mut float: f32) -> Rational64 {
    if !float.is_finite() || float > 100_000_000.0 {
        float = 0.0
    }
    let float_string = format!("{:.8}", float);
    let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
    let den = i64::pow(10, decimal.len() as u32);
    let num = i64::from_str(&float_string.replace('.', ""))
        .unwrap_or_else(|_| panic!("error converting {} to i64", float_string));

    Ratio::new(num, den)
}

fn eeg_datum_to_point_op(
    datum: f32,
    buffer: Option<&mut RingBuffer<f32>>,
    scale: f32,
    filename: &str,
) -> PointOp {
    let mut nameset = NameSet::new();
    nameset.insert(filename.into());
    let mut datum = datum.abs() * scale;
    if let Some(b) = buffer {
        b.push(datum);

        let b_vec = b.to_vec();
        let sum: f32 = b_vec.iter().sum();
        datum = sum / b_vec.len() as f32;
    }

    let fa = f32_to_rational(datum);
    PointOp {
        // fm,
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

fn get_data(filename: String) -> Result<Vec<CsvData>, Error> {
    let path = Path::new(&filename);
    let cwd = std::env::current_dir()?;
    let file = File::open(path).unwrap_or_else(|_| {
        panic!(
            "unable to read file: {}. current working directory is: {}",
            path.display(),
            cwd.display()
        )
    });
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(file);

    Ok(rdr
        .deserialize::<CsvData>()
        .map(|datum| datum.expect("Error deserializing datum"))
        .collect())
}
