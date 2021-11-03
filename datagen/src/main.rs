use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EEGData {
    data: Vec<f32>,
}

fn main() {
    let file_path = "data/sample_audvis_filt-0-40_raw_chanel_EEG_008_array_0.csv";
    let file = File::open(file_path).expect("unable to read file");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(file);
    let result: Vec<EEGData> = rdr.deserialize::<EEGData>().map(|t| t.unwrap()).collect();
    dbg!(&result[0]);
}
