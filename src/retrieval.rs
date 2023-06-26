use crate::long_term_memory::search_messages;
use crate::models::{AppState, SearchPayload};
use actix_web::{error, post, web, HttpResponse, Responder};
use std::ops::Deref;
use std::sync::Arc;

#[post("/sessions/{session_id}/retrieval")]
pub async fn run_retrieval(
    session_id: web::Path<String>,
    web::Json(payload): web::Json<SearchPayload>,
    data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    if !data.long_term_memory {
        return Ok(HttpResponse::BadRequest().body("Long term memory is disabled"));
    }

    let conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let client_wrapper = data.openai_pool.get().await.unwrap();
    let openai_client = client_wrapper.deref();

    match search_messages(payload.text, session_id.clone(), openai_client, conn).await {
        Ok(results) => Ok(HttpResponse::Ok().json(results)),
        Err(e) => {
            log::error!("Error Retrieval API: {:?}", e);
            Ok(HttpResponse::InternalServerError().body("Internal server error"))
        }
    }
}
