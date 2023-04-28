use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

mod healthcheck;
mod long_term_memory;
mod memory;
mod models;
mod redis_utils;
mod reducer;
mod retrieval;

use healthcheck::get_health;
use memory::{delete_memory, get_memory, post_memory};
use models::AppState;
use redis_utils::ensure_redisearch_index;
use retrieval::run_retrieval;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting Motörhead 🤘");

    let openai_client = async_openai::Client::new();
    let redis_url = env::var("REDIS_URL").expect("$REDIS_URL is not set");
    let redis = redis::Client::open(redis_url).unwrap();

    let long_term_memory = env::var("MOTORHEAD_LONG_TERM_MEMORY")
        .map(|value| value.to_lowercase() == "true")
        .unwrap_or(false);

    if long_term_memory {
        // TODO: Make these configurable - for now just ADA support
        let vector_dimensions = 1536;
        let distance_metric = "COSINE";

        ensure_redisearch_index(&redis, vector_dimensions, distance_metric).unwrap_or_else(|err| {
            eprintln!("RediSearch index error: {}", err);
            std::process::exit(1);
        });
    }

    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| 8000);

    let window_size = env::var("MOTORHEAD_MAX_WINDOW_SIZE")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| 12);

    let session_cleanup = Arc::new(Mutex::new(HashMap::new()));
    let session_state = Arc::new(AppState {
        window_size,
        session_cleanup,
        openai_client,
        long_term_memory,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis.clone()))
            .app_data(web::Data::new(session_state.clone()))
            .wrap(middleware::Logger::default())
            .service(get_health)
            .service(get_memory)
            .service(post_memory)
            .service(delete_memory)
            .service(run_retrieval)
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error":"{}"}}"#, err)),
                )
                .into()
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
