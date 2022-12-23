use crate::{Axis, NameSet, NormalForm, Normalize, Op, OscType, PointOp, Term, ASR};
use num_rational::{Ratio, Rational64};
use peekread::{PeekRead, SeekPeekReader};
use scop::Defs;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::str::FromStr;
use std::{fs::File, path::Path};
use weresocool_error::Error;
use weresocool_ring_buffer::RingBuffer;
use weresocool_shared::helpers::r_to_f32;
mod csv_test;
mod test;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub struct Point {
    fa: f32,
    t: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CsvData {
    data: Vec<f32>,
}

// #[derive(Debug)]
// struct Buffers<'a> {
// fa: Option<&'a mut RingBuffer<f32>>,
// lm: Option<&'a mut RingBuffer<f32>>,
// }

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Scale {
    pub axis: Axis,
    pub value: Rational64,
}

impl Scale {
    fn apply(&self, value: f32) -> Rational64 {
        self.value * f32_to_rational(value)
    }
}

pub fn csv_to_normalform(filename: &str, scales: Vec<Scale>) -> Result<NormalForm, Error> {
    let data = get_data(filename.into())?;
    let path = Path::new(&filename);
    Ok(csv_data_to_normal_form(
        &data,
        scales,
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .as_str(),
    ))
}

fn csv_data_to_normal_form(data: &Vec<Vec<f32>>, scales: Vec<Scale>, filename: &str) -> NormalForm {
    let mut length_ratio = Rational64::new(0, 1);

    // let mut buffer = RingBuffer::<f32>::new(50);

    let point_ops: Vec<PointOp> = data
        .iter()
        .map(|value| {
            let op = point_to_point_op(
                value,
                // Buffers {
                // fa: Some(&mut buffer),
                // lm: None,
                // },
                &scales, filename,
            );
            length_ratio += op.l;
            op
        })
        .collect();
    dbg!(&point_ops);

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

fn point_to_point_op(
    point: &Vec<f32>,
    // buffers: Buffers,
    scales: &Vec<Scale>,
    filename: &str,
) -> PointOp {
    let mut nameset = NameSet::new();
    nameset.insert(filename.to_string());
    let result: Vec<Rational64> = scales
        .iter()
        .enumerate()
        .map(|(i, s)| s.apply(point[i]))
        .collect();

    let fa = result[0];
    // if let Some(b) = buffers.fa {
    // b.push(fa);

    // let b_vec = b.to_vec();
    // let sum: f32 = b_vec.iter().sum();
    // fa = sum / b_vec.len() as f32;
    // }

    let lm = result[1];

    // if point.lm.is_some() {
    // let lm_inner = point.lm.unwrap().abs() * scale;
    // if let Some(b) = buffers.lm {
    // b.push(lm_inner);

    // let b_vec = b.to_vec();
    // let sum: f32 = b_vec.iter().sum();
    // fa = sum / b_vec.len() as f32;
    // }
    // lm = f32_to_rational(lm_inner);
    // }

    // let fa = f32_to_rational(fa);
    PointOp {
        // fm,
        fm: Rational64::new(1, 1),
        fa,
        l: lm,
        // l: Rational64::new(2, 100),
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

fn get_data(filename: String) -> Result<Vec<Vec<f32>>, Error> {
    let path = Path::new(&filename);
    let cwd = std::env::current_dir()?;
    let file = File::open(path).unwrap_or_else(|_| {
        panic!(
            "unable to read file: {}. current working directory is: {}",
            path.display(),
            cwd.display()
        )
    });
    let mut file = SeekPeekReader::new(file);
    // let first_line = file.peek();
    // dbg!(first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(file);

    let deserialized: Vec<Vec<f32>> = rdr
        .deserialize::<Vec<f32>>()
        .map(|datum| datum.expect("Error deserializing datum"))
        .collect();

    Ok(deserialized)
}
