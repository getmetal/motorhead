use actix_web::{error, get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;


#[derive(Deserialize)]
pub struct MemoryMessage {
    message: String,
}

#[derive(Deserialize)]
pub struct MemoryMessages {
    messages: Vec<MemoryMessage>,
}

#[derive(Serialize)]
struct HealthCheckResponse {
    now: u128,
}

#[get("/sessions/{session_id}/memory")]
pub async fn get_memory(
    session_id: web::Path<String>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let res = redis::Cmd::lrange(&*session_id, 0, 10)
        .query_async::<_, Vec<String>>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(res))
}

#[post("/sessions/{session_id}/memory")]
pub async fn post_memory(
    session_id: web::Path<String>,
    web::Json(memory_messages): web::Json<MemoryMessages>,
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

    Ok(HttpResponse::Ok())
}

#[get("/")]
pub async fn healthcheck() -> actix_web::Result<impl Responder> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let res = HealthCheckResponse {
        now: ms,
    };

    Ok(web::Json(res))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    let redis_url = env::var("REDIS_URL").expect("$REDIS_URL is not set");
    let redis = redis::Client::open(redis_url).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis.clone()))
            .wrap(middleware::Logger::default())
            .service(healthcheck)
            .service(get_memory)
            .service(post_memory)
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
