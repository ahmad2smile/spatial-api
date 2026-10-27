use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::sensor::{
    models::{SensorDto, SensorNearbyDto, SensorUpdateDto},
    service::SensorService,
};

pub fn sensor_api(pool: PgPool) -> Router {
    Router::new()
        .route(
            "/sensors",
            post(
                |State(srv): State<Arc<SensorService>>, Json(dto): Json<SensorDto>| async move {
                    srv.create(&dto).await.map(|_| StatusCode::CREATED)
                },
            ),
        )
        .route(
            "/sensors/{name}",
            get(
                |State(srv): State<Arc<SensorService>>, Path(name): Path<String>| async move {
                    srv.get(&name).await.map(Json)
                },
            )
            .put(
                |State(srv): State<Arc<SensorService>>,
                 Path(name): Path<String>,
                 Json(dto): Json<SensorUpdateDto>| async move {
                    srv.update(&name, &dto).await.map(|_| StatusCode::OK)
                },
            ),
        )
        .route("/sensors/nearby", post(
            |State(srv): State<Arc<SensorService>>,
            Json(dto): Json<SensorNearbyDto>|async move {
                srv.nearby(dto.lat, dto.long).await.map(Json)
            }))
        .with_state(Arc::new(SensorService::new(pool)))
}
