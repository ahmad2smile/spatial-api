use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::sensor::{
    models::{SensorDto, SensorNearbyDto, SensorUpdateDto},
    service::SensorService,
};

pub fn sensor_api(pool: PgPool) -> Router {
    Router::new()
        .route("/sensors", post(post_sensors))
        .route("/sensors/{name}", get(get_sensor).put(put_sensor))
        .route("/sensors/nearby", post(post_nearby))
        .with_state(Arc::new(SensorService::new(pool)))
}

#[utoipa::path(
        post,
        path = "/sensors",
        responses(
            (status = 201, description = "Sensor created"),
        )
    )]
pub async fn post_sensors(
    State(srv): State<Arc<SensorService>>,
    Json(dto): Json<SensorDto>,
) -> impl IntoResponse {
    srv.create(&dto).await.map(|_| StatusCode::CREATED)
}

#[utoipa::path(
        get,
        path = "/sensors/{name}",
        responses(
            (status = 200, description = "Sensor found"),
        ),
        params(
            ("name" = String, Path, description = "Unique sensor name"),
        )
    )]
pub async fn get_sensor(
    State(srv): State<Arc<SensorService>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    srv.get(&name).await.map(Json)
}

#[utoipa::path(
        put,
        path = "/sensors/{name}",
        responses(
            (status = 200, description = "Sensor updated"),
        ),
        params(
            ("name" = String, Path, description = "Unique sensor name"),
        )
    )]
pub async fn put_sensor(
    State(srv): State<Arc<SensorService>>,
    Path(name): Path<String>,
    Json(dto): Json<SensorUpdateDto>,
) -> impl IntoResponse {
    srv.update(&name, &dto).await.map(|_| StatusCode::OK)
}

#[utoipa::path(
        post,
        path = "/sensors/nearby",
        responses(
            (status = 200, description = "Nearby Sensor found"),
        )
    )]
pub async fn post_nearby(
    State(srv): State<Arc<SensorService>>,
    Json(dto): Json<SensorNearbyDto>,
) -> impl IntoResponse {
    srv.nearby(dto.lat, dto.long).await.map(Json)
}
