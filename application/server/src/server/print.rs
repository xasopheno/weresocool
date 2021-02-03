use crate::server::types::PrintLanguage;
use crate::server::Success;
use actix_web::{http::StatusCode, web, HttpResponse};
use serde::{Deserialize, Serialize};
use weresocool::generation::{RenderReturn, RenderType, WavType};
use weresocool::interpretable::{InputType, Interpretable};

#[derive(Deserialize, Serialize, Debug)]
pub struct PrintSuccess {
    audio: Vec<u8>,
    print_type: String,
}

impl PrintSuccess {
    pub fn new(audio: Vec<u8>, print_type: String) -> Self {
        Self { audio, print_type }
    }
}

pub async fn print(req: web::Json<PrintLanguage>) -> HttpResponse {
    let result = if req.print_type == "mp3".to_string() {
        InputType::Language(&req.language).make(RenderType::Wav(WavType::MP3 { cli: false }), None)
    } else {
        InputType::Language(&req.language).make(RenderType::Wav(WavType::Wav { cli: false }), None)
    };
    match result {
        Ok(render_return) => match render_return {
            RenderReturn::Wav(wav) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(Success::PrintSuccess(PrintSuccess::new(
                    wav,
                    req.print_type.to_owned(),
                ))),
            _ => panic!(),
        },
        Err(parse_error) => {
            let inner = *parse_error.inner;
            HttpResponse::Ok().json(inner.into_serializeable())
        }
    }
}
