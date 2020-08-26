use crate::server::types::PrintLanguage;
use crate::server::{PrintSuccess, Success};
use actix_web::{http::StatusCode, web, HttpResponse};
use weresocool::generation::{RenderReturn, RenderType, WavType};
use weresocool::interpretable::{InputType, Interpretable};
use weresocool_error::Error;

pub async fn print(req: web::Json<PrintLanguage>) -> HttpResponse {
    let result: Result<RenderReturn, Error> = match req.print_type.as_str() {
        "mp3" => InputType::Language(&req.language)
            .make(RenderType::Wav(WavType::MP3 { cli: false }), None),
        "wav" => InputType::Language(&req.language)
            .make(RenderType::Wav(WavType::MP3 { cli: false }), None),
        "csv" => InputType::Language(&req.language).make(RenderType::Csv1d, None),
        _ => Err(Error::with_msg(format!(
            "Backend Error. Cannot render type of {}.",
            req.print_type
        ))),
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
