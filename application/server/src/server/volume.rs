use crate::server::types::VolumeUpdate;
use crate::server::Success;
use actix_web::{web, HttpResponse};
use std::sync::{Arc, Mutex};
use weresocool::manager::RenderManager;

pub async fn volume_update(
    render_manager: web::Data<Arc<Mutex<RenderManager>>>,
    req: web::Json<VolumeUpdate>,
) -> HttpResponse {
    render_manager.lock().unwrap().update_volume(req.volume);
    println!("Volume: {}", req.volume);
    HttpResponse::Ok().json(Success::VolumeUpdate(format!(
        "VolumeUpdate: {}",
        req.volume
    )))
}
