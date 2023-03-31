use actix_web::{delete, error, get, post, web, HttpResponse, Responder};
use log::info;
use std::convert::TryInto;
use std::sync::Arc;
use tokio;

use crate::models::{AppState, MemoryMessages, MemoryResponse, SessionState};

#[get("/sessions/{session_id}/memory")]
pub async fn get_memory(
    session_id: web::Path<String>,
    data: web::Data<AppState>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let lrange_key = &*session_id;
    let get_key = format!("{}_context", &*session_id);

    let (lrange_res, get_res): (Vec<String>, Option<String>) = redis::pipe()
        .cmd("LRANGE")
        .arg(lrange_key)
        .arg(0)
        .arg(data.window_size as isize)
        .cmd("GET")
        .arg(get_key)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let response = MemoryResponse {
        messages: lrange_res,
        context: get_res,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/sessions/{session_id}/memory")]
pub async fn post_memory(
    session_id: web::Path<String>,
    web::Json(memory_messages): web::Json<MemoryMessages>,
    data: web::Data<AppState>,
    session_state: web::Data<Arc<SessionState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let messages: Vec<String> = memory_messages
        .messages
        .into_iter()
        .map(|memory_message| memory_message.message)
        .collect();

    let res: i64 = redis::Cmd::lpush(&*session_id, messages)
        .query_async::<_, i64>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    info!("{}", format!("Redis response, {}", res));

    if res > data.window_size {
        let state = session_state.into_inner();
        let mut cleaning_up = state.cleaning_up.lock().await;

        if !cleaning_up.get(&*session_id).unwrap_or_else(|| &false) {
            info!("Window size bigger!2");

            cleaning_up.insert((&*session_id.to_string()).into(), true);
            let cleaning_up = Arc::clone(&state.cleaning_up);
            let session_id = session_id.to_string().clone();

            tokio::spawn(async move {
                // Summarization
                // Sumarize entire thing?
                // How do we see retrieving information outside of the Chat History.

                info!("Inside job");
                let half = &data.window_size / 2;
                let res = redis::Cmd::lrange(
                    &*session_id,
                    half.try_into().unwrap(),
                    data.window_size.try_into().unwrap(),
                )
                .query_async::<_, Vec<String>>(&mut conn)
                .await;

                info!("{:?}", res);

                let mut lock = cleaning_up.lock().await;
                lock.remove(&session_id);
            });
        }
    }

    Ok(HttpResponse::Ok())
}

#[delete("/sessions/{session_id}/memory")]
pub async fn delete_memory(
    session_id: web::Path<String>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    redis::Cmd::del(&*session_id)
        .query_async::<_, i64>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}
