use actix_web::{delete, error, get, post, web, HttpResponse, Responder};
// use log::{error, info};
use std::convert::TryInto;
use std::sync::Arc;
use tokio;

use crate::models::{AckResponse, AppState, MemoryMessages, MemoryResponse};
use crate::reducer::incremental_summarization;

#[get("/sessions/{session_id}/memory")]
pub async fn get_memory(
    session_id: web::Path<String>,
    data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let lrange_key = &*session_id;
    let context_key = format!("{}_context", &*session_id);

    let (messages, context): (Vec<String>, Option<String>) = redis::pipe()
        .cmd("LRANGE")
        .arg(lrange_key)
        .arg(0)
        .arg(data.window_size as isize)
        .cmd("GET")
        .arg(context_key)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let response = MemoryResponse { messages, context };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}

#[post("/sessions/{session_id}/memory")]
pub async fn post_memory(
    session_id: web::Path<String>,
    web::Json(memory_messages): web::Json<MemoryMessages>,
    data: web::Data<Arc<AppState>>,
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

    if res > data.window_size {
        let state = data.into_inner();
        let mut session_cleanup = state.session_cleanup.lock().await;

        if !session_cleanup.get(&*session_id).unwrap_or_else(|| &false) {
            session_cleanup.insert((&*session_id.to_string()).into(), true);
            let session_cleanup = Arc::clone(&state.session_cleanup);
            let session_id = session_id.to_string().clone();
            let state_clone = state.clone();

            tokio::spawn(async move {
                let half = state_clone.window_size / 2;
                log::info!("{}", format!("Inside job, {}, {}", half, state_clone.window_size));
                let context_key = format!("{}_context", &*session_id);
                let (messages, context): (Vec<String>, Option<String>) = redis::pipe()
                    .cmd("LRANGE")
                    .arg(&*session_id)
                    .arg(TryInto::<i64>::try_into(half).unwrap())
                    .arg(TryInto::<i64>::try_into(state_clone.window_size).unwrap())
                    .cmd("GET")
                    .arg(context_key.clone())
                    .query_async(&mut conn)
                    .await
                    .unwrap();

                let new_context_result =
                    incremental_summarization(state_clone.openai_client.clone(), context, messages)
                        .await;

                if let Err(ref error) = new_context_result {
                    log::error!("Problem getting summary: {:?}", error);
                }

                let new_context = new_context_result.unwrap_or_default();

                let redis_pipe_response_result: Result<((), ()), redis::RedisError> = redis::pipe()
                    .cmd("LTRIM")
                    .arg(&*session_id)
                    .arg(0)
                    .arg(TryInto::<i64>::try_into(half).unwrap())
                    .cmd("SET")
                    .arg(context_key)
                    .arg(new_context)
                    .query_async(&mut conn)
                    .await;

                match redis_pipe_response_result {
                    Ok(_) => (),
                    Err(e) => log::error!("Error executing the redis pipeline: {:?}", e),
                }

                let mut lock = session_cleanup.lock().await;
                lock.remove(&session_id);
            });
        }
    }

    let response = AckResponse { status: "Ok" };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
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

    let context_key = format!("{}_context", &*session_id);

    redis::pipe()
        .cmd("DEL")
        .arg(&*session_id)
        .cmd("DEL")
        .arg(context_key)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let response = AckResponse { status: "Ok" };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}
