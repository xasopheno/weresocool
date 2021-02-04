use crate::server::types::PrintLanguage;
use crate::server::Success;
use actix_web::{http::StatusCode, web, HttpResponse};
use serde::{Deserialize, Serialize};
use weresocool::generation::{RenderReturn, RenderType, WavType};
use weresocool::interpretable::{InputType, Interpretable};

#[derive(Deserialize, Serialize, Debug)]
pub struct StemSuccess {
    stems: Vec<Vec<u8>>,
    print_type: String,
}

impl StemSuccess {
    pub fn new(stems: Vec<Vec<u8>>, print_type: String) -> Self {
        Self { stems, print_type }
    }
}

pub async fn stems(req: web::Json<PrintLanguage>) -> HttpResponse {
    let result = InputType::Language(&req.language).make(RenderType::Stems, None);
    match result {
        Ok(render_return) => match render_return {
            RenderReturn::Stems(stems) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(Success::StemSuccess(StemSuccess::new(
                    stems,
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
