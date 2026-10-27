use std::time::Duration;

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;

use crate::sensor::api::sensor_api;

mod sensor;
mod utils;

#[tokio::main]
async fn main() {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    // build our application with a single route
    let app = Router::new()
        .route("/healthz", get(|| async { "Healthy" }))
        .merge(sensor_api(pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
