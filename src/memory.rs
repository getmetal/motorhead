use actix_web::{delete, error, get, post, web, HttpResponse, Responder};
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;

pub struct AppState {
    pub window_size: i64,
}

pub struct SessionState {
    pub cleaning_up: Arc<Mutex<HashMap<String, bool>>>,
}

#[derive(Deserialize)]
pub struct MemoryMessage {
    message: String,
}

#[derive(Deserialize)]
pub struct MemoryMessages {
    messages: Vec<MemoryMessage>,
}

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

    let res = redis::Cmd::lrange(&*session_id, 0, data.window_size.try_into().unwrap())
        .query_async::<_, Vec<String>>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(res))
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
        let mut cleaning_up = state.cleaning_up.lock().await; // Use lock().await

        if !cleaning_up.get(&*session_id).unwrap_or_else(|| &false) {
            info!("Window size bigger!2");

            cleaning_up.insert((&*session_id.to_string()).into(), true);
            let cleaning_up = Arc::clone(&state.cleaning_up);
            let session_id = session_id.to_string().clone();

            tokio::spawn(async move {
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
