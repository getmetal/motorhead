use crate::long_term_memory::index_messages;
use crate::models::{
    AckResponse, AppState, GetSessionsQuery, MemoryMessage, MemoryMessagesAndContext,
    MemoryResponse, NamespaceQuery,
};
use crate::reducer::handle_compaction;
use actix_web::{delete, error, get, post, web, HttpResponse, Responder};
use std::ops::Deref;
use std::sync::Arc;

#[get("/sessions")]
pub async fn get_sessions(
    web::Query(pagination): web::Query<GetSessionsQuery>,
    _data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let GetSessionsQuery {
        page,
        size,
        namespace,
    } = pagination;

    if page > 100 {
        return Err(actix_web::error::ErrorBadRequest(
            "Page size must not exceed 100",
        ));
    }

    let start: isize = ((page - 1) * size) as isize; // 0-indexed
    let end: isize = (page * size - 1) as isize; // inclusive

    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let sessions_key = match &namespace {
        Some(namespace) => format!("sessions:{}", namespace),
        None => String::from("sessions"),
    };

    let session_ids: Vec<String> = redis::cmd("ZRANGE")
        .arg(sessions_key)
        .arg(start)
        .arg(end)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(session_ids))
}

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

    let lrange_key = format!("session:{}", &*session_id);
    let context_key = format!("context:{}", &*session_id);
    let token_count_key = format!("tokens:{}", &*session_id);
    let keys = vec![context_key, token_count_key];

    let (messages, values): (Vec<String>, Vec<Option<String>>) = redis::pipe()
        .cmd("LRANGE")
        .arg(lrange_key)
        .arg(0)
        .arg(data.window_size as isize)
        .cmd("MGET")
        .arg(keys)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let context = values.get(0).cloned().flatten();
    let tokens = values
        .get(1)
        .cloned()
        .flatten()
        .and_then(|tokens_string| tokens_string.parse::<i64>().ok())
        .unwrap_or(0);

    let messages: Vec<MemoryMessage> = messages
        .into_iter()
        .filter_map(|message| {
            let mut parts = message.splitn(2, ": ");
            match (parts.next(), parts.next()) {
                (Some(role), Some(content)) => Some(MemoryMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                }),
                _ => None,
            }
        })
        .collect();

    let response = MemoryResponse {
        messages,
        context,
        tokens: Some(tokens),
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}

#[post("/sessions/{session_id}/memory")]
pub async fn post_memory(
    session_id: web::Path<String>,
    web::Json(memory_messages): web::Json<MemoryMessagesAndContext>,
    data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
    web::Query(namespace_query): web::Query<NamespaceQuery>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let memory_messages_clone: Vec<MemoryMessage> = memory_messages.messages.to_vec();

    let messages: Vec<String> = memory_messages
        .messages
        .into_iter()
        .map(|memory_message| format!("{}: {}", memory_message.role, memory_message.content))
        .collect();

    // If new context is passed in we overwrite the existing one
    if let Some(context) = memory_messages.context {
        redis::Cmd::set(format!("context:{}", &*session_id), context)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }

    let sessions_key = match namespace_query.namespace {
        Some(namespace) => format!("sessions:{}", namespace),
        None => String::from("sessions"),
    };

    // add to sorted set of sessions
    redis::cmd("ZADD")
        .arg(sessions_key)
        .arg(chrono::Utc::now().timestamp())
        .arg(&*session_id)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let res: i64 = redis::Cmd::lpush(format!("session:{}", &*session_id), messages.clone())
        .query_async::<_, i64>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if data.long_term_memory {
        let session = session_id.clone();
        let conn_clone = conn.clone();
        let pool = data.openai_pool.clone();

        tokio::spawn(async move {
            let client_wrapper = pool.get().await.unwrap();
            let client = client_wrapper.deref();
            if let Err(e) = index_messages(memory_messages_clone, session, client, conn_clone).await
            {
                log::error!("Error in index_messages: {:?}", e);
            }
        });
    }

    if res > data.window_size {
        let state = data.into_inner();
        let mut session_cleanup = state.session_cleanup.lock().await;

        if !session_cleanup.get(&*session_id).unwrap_or(&false) {
            session_cleanup.insert((&*session_id.to_string()).into(), true);
            let session_cleanup = Arc::clone(&state.session_cleanup);
            let session_id = session_id.clone();
            let window_size = state.window_size;
            let model = state.model.to_string();
            let pool = state.openai_pool.clone();

            tokio::spawn(async move {
                log::info!("running compact");
                let client_wrapper = pool.get().await.unwrap();
                let client = client_wrapper.deref();

                let _compaction_result =
                    handle_compaction(session_id.to_string(), model, window_size, client, conn)
                        .await;

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
    web::Query(namespace_query): web::Query<NamespaceQuery>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let context_key = format!("context:{}", &*session_id);
    let token_count_key = format!("tokens:{}", &*session_id);
    let session_key = format!("session:{}", &*session_id);
    let keys = vec![context_key, session_key, token_count_key];

    let sessions_key = match namespace_query.namespace {
        Some(namespace) => format!("sessions:{}", namespace),
        None => String::from("sessions"),
    };

    redis::cmd("ZREM")
        .arg(sessions_key)
        .arg(&*session_id)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    redis::Cmd::del(keys)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let response = AckResponse { status: "Ok" };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}
