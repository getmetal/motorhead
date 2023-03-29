use actix_web::{middleware, web, App, HttpServer};
use std::env;
use std::io;

mod memory;
use memory::{delete_memory, get_memory, post_memory};

mod healthcheck;
use healthcheck::get_health;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting server");

    let redis_url = env::var("REDIS_URL").expect("$REDIS_URL is not set");
    let redis = redis::Client::open(redis_url).unwrap();
    let port = env::var("PORT")
        .unwrap_or_else(|_| String::from("8080"))
        .parse::<u16>()
        .unwrap_or_else(|_| 8080);

    let window_size = env::var("WINDOW_SIZE")
        .unwrap_or_else(|_| String::from("10"))
        .parse::<i64>()
        .unwrap_or_else(|_| 10);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis.clone()))
            .app_data(web::Data::new(window_size.clone()))
            .wrap(middleware::Logger::default())
            .service(get_health)
            .service(get_memory)
            .service(post_memory)
            .service(delete_memory)
    })
    .workers(2)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
