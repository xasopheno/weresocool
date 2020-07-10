use crate::server::types::PrintLanguage;
use crate::server::{PrintSuccess, Success};
// use actix_cors::Cors;
use actix_web::{http::StatusCode, web, HttpResponse};
use weresocool::generation::{RenderReturn, RenderType, WavType};
use weresocool::interpretable::{InputType, Interpretable};

pub async fn print(req: web::Json<PrintLanguage>) -> HttpResponse {
    let result = if req.print_type == "mp3".to_string() {
        InputType::Language(&req.language).make(RenderType::Wav(WavType::MP3 { cli: false }))
    } else {
        InputType::Language(&req.language).make(RenderType::Wav(WavType::Wav { cli: false }))
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
