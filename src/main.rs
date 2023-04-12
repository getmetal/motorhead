use std::io;
mod healthcheck;
mod long_term_memory;
mod memory;
mod models;
mod reducer;
mod retrieval;
mod server;
use server::start_app;

#[actix_web::main]
async fn main() -> io::Result<()> {
    start_app().await
}
