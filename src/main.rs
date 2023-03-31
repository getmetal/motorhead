use actix_web::{middleware, web, App, HttpServer};
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

mod memory;
use memory::{delete_memory, get_memory, post_memory};
mod models;
use models::{AppState, SessionState};
mod healthcheck;
use healthcheck::get_health;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting server");

    let openai_key = env::var("OPENAI_API_KEY").unwrap_or("NOT_SET".to_string());
    let reduce_method = env::var("WINDOW_REDUCE_METHOD").unwrap_or("buffer".to_string());

    if reduce_method == "summarization" && openai_key == "NOT_SET" {
        panic!("`OPENAI_API_KEY` is required if `summarization` is the `WINDOW_REDUCE_METHOD`");
    }

    let redis_url = env::var("REDIS_URL").expect("$REDIS_URL is not set");
    let redis = redis::Client::open(redis_url).unwrap();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or_else(|_| 8080);

    let window_size = env::var("WINDOW_SIZE")
        .unwrap_or_else(|_| String::from("10"))
        .parse::<i64>()
        .unwrap_or_else(|_| 10);

    let cleaning_up = Arc::new(Mutex::new(HashMap::new()));
    let session_state = Arc::new(SessionState {
        cleaning_up,
        openai_key,
        reduce_method,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis.clone()))
            .app_data(web::Data::new(AppState {
                window_size: window_size,
            }))
            .app_data(web::Data::new(window_size.clone()))
            .app_data(web::Data::new(session_state.clone()))
            .wrap(middleware::Logger::default())
            .service(get_health)
            .service(get_memory)
            .service(post_memory)
            .service(delete_memory)
    })
    // .workers(2)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
