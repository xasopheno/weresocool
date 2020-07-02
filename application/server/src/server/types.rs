use serde::{Deserialize, Serialize};
use weresocool::instrument::StereoWaveform;
use weresocool_error::ParseError;

#[derive(Deserialize, Serialize, Debug)]
pub enum RenderResponse {
    RenderSuccess(),
    RenderError(),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Language {
    pub language: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RenderSuccess {
    pub response_type: RenderResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RenderError {
    pub response_type: RenderResponse,
    pub error: ParseError,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PrintSuccess {
    pub response_type: RenderResponse,
    pub wav: Vec<u8>,
}

impl PrintSuccess {
    pub const fn new(wav: Vec<u8>) -> Self {
        Self {
            response_type: RenderResponse::RenderSuccess(),
            wav,
        }
    }
}
