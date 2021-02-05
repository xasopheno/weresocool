use crate::server::types::StemLanguage;
use crate::server::Success;
use actix_web::{http::StatusCode, web, HttpResponse};
use serde::{Deserialize, Serialize};
use weresocool::generation::{RenderReturn, RenderType, Stem};
use weresocool::interpretable::{InputType, Interpretable};

#[derive(Deserialize, Serialize, Debug)]
pub struct StemsSuccess {
    stems: Vec<Stem>,
    print_type: String,
}

impl StemsSuccess {
    pub fn new(stems: Vec<Stem>, print_type: String) -> Self {
        Self { stems, print_type }
    }
}

pub async fn stems(req: web::Json<StemLanguage>) -> HttpResponse {
    let result = InputType::Language(&req.language).make(RenderType::Stems, None);
    match result {
        Ok(render_return) => match render_return {
            RenderReturn::Stems(stems) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(Success::StemsSuccess(StemsSuccess::new(
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
