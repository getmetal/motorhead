use actix_web::{get, web, Responder};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct HealthCheckResponse {
    now: u128,
}

#[get("/")]
pub async fn get_health() -> actix_web::Result<impl Responder> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let res = HealthCheckResponse { now: ms };

    Ok(web::Json(res))
}
