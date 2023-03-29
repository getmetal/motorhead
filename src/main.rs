use std::env;
use std::io;

use actix_web::{error, get, middleware, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MemoryMessage {
    message: String,
}

async fn create_memory(
    web::Json(info): web::Json<MemoryMessage>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let res = redis::Cmd::lpush("foo", &info.message)
        .query_async::<_, String>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // not strictly necessary, but successful SET operations return "OK"
    if res == "OK" {
        Ok(HttpResponse::Ok().body("successfully cached values"))
    } else {
        Ok(HttpResponse::InternalServerError().finish())
    }
}

async fn get_memory(redis: web::Data<redis::Client>) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let res = redis::Cmd::lrange("foo", 0, 0)
        .query_async::<_, String>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // not strictly necessary, but successful SET operations return "OK"
    if res == "OK" {
        Ok(HttpResponse::Ok().body("successfully cached values"))
    } else {
        Ok(HttpResponse::InternalServerError().finish())
    }
}

async fn del_stuff(redis: web::Data<redis::Client>) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let res = redis::Cmd::del(&["my_domain:one", "my_domain:two", "my_domain:three"])
        .query_async::<_, usize>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // not strictly necessary, but successful DEL operations return the number of keys deleted
    if res == 3 {
        Ok(HttpResponse::Ok().body("successfully deleted values"))
    } else {
        log::error!("deleted {res} keys");
        Ok(HttpResponse::InternalServerError().finish())
    }
}

#[get("/users/{user_id}")]
async fn get_user(user_id: web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User ID: {}", user_id))
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
            .service(get_user)
            .service(
                web::resource("/memory")
                    .route(web::post().to(create_memory))
                    .route(web::get().to(get_memory))
                    .route(web::delete().to(del_stuff)),
            )
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
