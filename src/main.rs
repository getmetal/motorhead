use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

mod memory;
mod reducer;
use memory::{delete_memory, get_memory, post_memory};
mod models;
use models::AppState;
mod healthcheck;
use healthcheck::get_health;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting Motörhead 🤘");

    let openai_client = async_openai::Client::new();
    let redis_url = env::var("REDIS_URL").expect("$REDIS_URL is not set");
    let redis = redis::Client::open(redis_url).unwrap();
    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| 8000);

    let window_size = env::var("MAX_WINDOW_SIZE")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| 12);

    let session_cleanup = Arc::new(Mutex::new(HashMap::new()));
    let session_state = Arc::new(AppState {
        window_size,
        session_cleanup,
        openai_client,
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
