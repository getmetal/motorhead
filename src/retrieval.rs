use crate::long_term_memory::search_messages;
use crate::models::{AppState, SearchPayload};
use actix_web::{error, post, web, HttpResponse, Responder};
use std::sync::Arc;

#[post("/sessions/{session_id}/retrieval")]
pub async fn run_retrieval(
    session_id: web::Path<String>,
    web::Json(payload): web::Json<SearchPayload>,
    data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let openai_client = data.openai_client.clone();

    match search_messages(payload.text, session_id.clone(), openai_client, conn).await {
        Ok(results) => Ok(HttpResponse::Ok().json(results)),
        Err(e) => {
            log::error!("Error Retrieval API: {:?}", e);
            Ok(HttpResponse::InternalServerError().body("Internal server error"))
        }
    }
}
