use crate::server::types::PrintLanguage;
use crate::server::{DataSuccess, PrintSuccess, Success};
use actix_web::{http::StatusCode, web, HttpResponse};
use weresocool::generation::{RenderReturn, RenderType, WavType};
use weresocool::interpretable::{InputType, Interpretable};

pub async fn print(req: web::Json<PrintLanguage>) -> HttpResponse {
    let result = match req.print_type.as_str() {
        "mp3" => InputType::Language(&req.language)
            .make(RenderType::Wav(WavType::MP3 { cli: false }), None),
        "wav" => InputType::Language(&req.language)
            .make(RenderType::Wav(WavType::Wav { cli: false }), None),
        "csv" => InputType::Language(&req.language).make(RenderType::Csv1d, None),
        "json" => InputType::Language(&req.language).make(RenderType::Json4d { cli: false }, None),
        _ => unimplemented!(),
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
            RenderReturn::Csv1d(csv) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(Success::DataSuccess(DataSuccess::new(
                    csv,
                    req.print_type.to_owned(),
                ))),
            RenderReturn::Json4d(json) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(Success::DataSuccess(DataSuccess::new(
                    json,
                    req.print_type.to_owned(),
                ))),
            _ => unimplemented!(),
        },
        Err(parse_error) => {
            let inner = *parse_error.inner;
            HttpResponse::Ok().json(inner.into_serializeable())
        }
    }
}
