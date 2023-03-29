use actix_web::{delete, error, get, post, web, HttpResponse, Responder};
use log::info;
use serde::Deserialize;

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
    window_size: web::Data<i64>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let res = redis::Cmd::lrange(&*session_id, 0, (**window_size).try_into().unwrap())
        .query_async::<_, Vec<String>>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(res))
}

#[post("/sessions/{session_id}/memory")]
pub async fn post_memory(
    session_id: web::Path<String>,
    web::Json(memory_messages): web::Json<MemoryMessages>,
    window_size: web::Data<i64>,
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

    if res > **window_size {
        info!("Window size bigger!");
    }

    Ok(HttpResponse::Ok())
}

#[delete("/sessions/{session_id}/memory")]
pub async fn delete_memory(
    session_id: web::Path<String>,
    window_size: web::Data<i64>,
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
