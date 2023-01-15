use crate::{Axis, NameSet, Op, Term};
use num_rational::{Ratio, Rational64};
use serde::{Deserialize, Serialize};
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

pub fn csv1d_to_normalform(filename: &str, scales: Vec<Scale>) -> Result<Term, Error> {
    let data = get_data1d(filename.into(), scales[1].value)?;
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

pub fn csv2d_to_normalform(filename: &str, scales: Vec<Scale>) -> Result<Term, Error> {
    let data = get_data2d(filename.into())?;
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

fn csv_data_to_normal_form(data: &[Vec<f32>], scales: Vec<Scale>, filename: &str) -> Term {
    // let mut buffer = RingBuffer::<f32>::new(50);

    let point_ops: Vec<Term> = data
        .iter()
        .map(|value| {
            point_to_point_op(
                value, None, // Some(&mut buffer),
                &scales, filename,
            )
        })
        .collect();

    Term::Op(Op::Sequence {
        operations: point_ops,
    })
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
    point: &[f32],
    buffer: Option<&mut RingBuffer<f32>>,
    scales: &[Scale],
    filename: &str,
) -> Term {
    let mut nameset = NameSet::new();
    nameset.insert(filename.to_string());
    let result: Vec<f32> = scales.iter().enumerate().map(|(i, _s)| point[i]).collect();

    let mut fa = result[0];
    if let Some(b) = buffer {
        b.push(fa);

        let b_vec = b.to_vec();
        let sum: f32 = b_vec.iter().sum();
        fa = sum / b_vec.len() as f32;
    }

    let lm = result[1];

    Term::Op(Op::Compose {
        operations: vec![
            Term::Op(Op::TransposeA {
                a: scales[0].apply(fa),
            }),
            Term::Op(Op::Length {
                m: f32_to_rational(lm),
            }),
        ],
    })
}

fn get_data1d(filename: String, length: Rational64) -> Result<Vec<Vec<f32>>, Error> {
    let length = r_to_f32(length);
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

    let deserialized: Vec<Vec<f32>> = rdr
        .deserialize::<Vec<f32>>()
        .map(|datum| datum.expect("Error deserializing datum"))
        .collect();
    dbg!(&deserialized);
    let result: Vec<Vec<f32>> = deserialized[0].iter().map(|v| vec![*v, length]).collect();
    dbg!(&result);

    Ok(result)
}

fn get_data2d(filename: String) -> Result<Vec<Vec<f32>>, Error> {
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

    let deserialized: Vec<Vec<f32>> = rdr
        .deserialize::<Vec<f32>>()
        .map(|datum| datum.expect("Error deserializing datum"))
        .collect();

    Ok(deserialized)
}
