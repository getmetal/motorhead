use crate::long_term_memory::search;
use crate::models::{AppState, SearchInput};
use actix_web::{post, web, HttpResponse, Responder};
use std::sync::Arc;

#[post("/sessions/{session_id}/retrieval")]
pub async fn run_retrieval(
    web::Json(payload): web::Json<SearchInput>,
    data: web::Data<Arc<AppState>>,
) -> impl Responder {
    let api_key = data.metal_secret.clone();
    let client_id = data.metal_client_id.clone();
    let app_id = data.metal_app_id.clone();

    match search(payload.text, app_id, api_key, client_id).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            log::error!("Error Retrieval API: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}
