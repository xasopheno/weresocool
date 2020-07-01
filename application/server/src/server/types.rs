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
    pub buffers: StereoWaveform,
}

impl PrintSuccess {
    pub const fn new(buffers: StereoWaveform) -> Self {
        Self {
            response_type: RenderResponse::RenderSuccess(),
            buffers,
        }
    }
}
