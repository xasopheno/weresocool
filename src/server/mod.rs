mod types;
use crate::generation::{RenderReturn, RenderType};
use crate::interpretable::{InputType, Interpretable};
use crate::server::types::{Language, RenderError, RenderSuccess};
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer};
use std::path::PathBuf;
use weresocool_error::ErrorInner;

async fn get_file(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let mut path = PathBuf::from("./renders");
    let file: PathBuf = req.match_info().query("filename").parse().unwrap();
    path.push(file);
    Ok(NamedFile::open(path)?)
}

async fn single_page_app(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path = PathBuf::from("./src/server/build/index.html");
    Ok(NamedFile::open(path)?)
}

async fn render(req: web::Json<Language>) -> HttpResponse {
    let result = InputType::Language(&req.language).make(RenderType::StereoWaveform);
    match result {
        Ok(render_return) => match render_return {
            RenderReturn::StereoWaveform(wav) => HttpResponse::Ok()
                .content_type("application/json")
                .status(StatusCode::OK)
                .json(RenderSuccess::new(wav)),
            _ => panic!(),
        },
        Err(parse_error) => {
            let inner = *parse_error.inner;
            match inner {
                ErrorInner::ParseError(error) => HttpResponse::Ok()
                    .content_type("application/json")
                    .status(StatusCode::OK)
                    .json(RenderError::new(error)),
                _ => panic!(),
            }
        }
    }
}

//#[actix_rt::main]
//pub async fn run() -> std::io::Result<()> {
//std::env::set_var("RUST_LOG", "socool_server=info,actix_web=info");
//env_logger::init();
//server().await
//}

pub async fn server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::new().finish())
            .service(
                web::scope("/api")
                    .route("/render", web::post().to(render))
                    .route("/songs/{filename:.*}", web::get().to(get_file)),
            )
            .route("/compose", web::get().to(single_page_app))
            .route("/play/{filename:.*}", web::get().to(single_page_app))
            .default_service(Files::new("/", "./src/server/build").index_file("index.html"))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test, web, App};

    #[actix_rt::test]
    async fn test_index() {
        let language = Language {
            language: "{f: 100, l: 1, g: 1, p: 0}\nmain={Tm 1}\n".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/api/render")
            .header(header::CONTENT_TYPE, "application/json")
            .set_json(&language)
            .to_request();

        let mut app = test::init_service(
            App::new().service(web::resource("/api/render").route(web::post().to(render))),
        )
        .await;

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
