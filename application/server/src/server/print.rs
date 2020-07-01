use crate::server::types::{Language, PrintSuccess};
// use actix_cors::Cors;
use actix_web::{http::StatusCode, web, HttpResponse};
use weresocool::generation::{RenderReturn, RenderType};
use weresocool::interpretable::{InputType, Interpretable};

pub async fn print(req: web::Json<Language>) -> HttpResponse {
    let result = InputType::Language(&req.language).make(RenderType::Wav);
    match result {
        Ok(render_return) => match render_return {
            RenderReturn::StereoWaveform(wav) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(PrintSuccess::new(wav)),
            _ => panic!(),
        },
        Err(parse_error) => {
            let inner = *parse_error.inner;
            HttpResponse::Ok().json(inner.into_serializeable())
        }
    }
}
