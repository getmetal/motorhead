mod healthcheck;
mod long_term_memory;
mod memory;
mod models;
mod redis_utils;
mod reducer;
mod retrieval;

use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use healthcheck::get_health;
use memory::{delete_memory, get_memory, get_sessions, post_memory};
use models::{AppState, OpenAIClientManager};
use redis_utils::ensure_redisearch_index;
use retrieval::run_retrieval;
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting Motorhead 🤘");

    let manager = OpenAIClientManager {};
    let max_size = 8;
    let openai_pool = deadpool::managed::Pool::builder(manager)
        .max_size(max_size)
        .build()
        .unwrap();

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
        .unwrap_or(8000);

    let window_size = env::var("MOTORHEAD_MAX_WINDOW_SIZE")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(12);
    let model = env::var("MOTORHEAD_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

    let session_cleanup = Arc::new(Mutex::new(HashMap::new()));
    let session_state = Arc::new(AppState {
        window_size,
        session_cleanup,
        openai_pool,
        long_term_memory,
        model,
    });

    async fn on_start_logger(port: u16) -> io::Result<()> {
        println!();
        println!("-----------------------------------");
        println!("🧠 Motorhead running on port: {}", port);
        println!("-----------------------------------");
        println!();

        Ok(())
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis.clone()))
            .app_data(web::Data::new(session_state.clone()))
            .wrap(middleware::Logger::default())
            .service(get_health)
            .service(get_memory)
            .service(post_memory)
            .service(delete_memory)
            .service(get_sessions)
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
    .run();

    let server_future = server;
    let logger_future = on_start_logger(port);

    let (server_result, logger_result) = tokio::join!(server_future, logger_future);

    // Handle the Result from server
    if let Err(e) = server_result {
        eprintln!("Server error: {}", e);
    }

    // Handle the Result from logger
    if let Err(e) = logger_result {
        eprintln!("Logger error: {}", e);
    }

    Ok(())
}
