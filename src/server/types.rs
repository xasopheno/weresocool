use crate::instrument::StereoWaveform;
use serde::{Deserialize, Serialize};
use weresocool_error::ParseError;

#[derive(Deserialize, Serialize, Debug)]
pub enum RenderResponse {
    RenderSuccess,
    RenderError,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Language {
    pub language: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RenderSuccess {
    pub response_type: RenderResponse,
    pub buffers: StereoWaveform,
}

impl RenderSuccess {
    pub const fn new(buffers: StereoWaveform) -> Self {
        Self {
            response_type: RenderResponse::RenderSuccess,
            buffers,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RenderError {
    pub response_type: RenderResponse,
    pub error: ParseError,
}

impl RenderError {
    pub const fn new(error: ParseError) -> Self {
        Self {
            response_type: RenderResponse::RenderError,
            error,
        }
    }
}
