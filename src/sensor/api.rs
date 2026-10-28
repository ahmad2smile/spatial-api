use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sqlx::PgPool;
use validator::ValidateLength;

use crate::{
    sensor::{
        models::{SensorDto, SensorNearbyDto, SensorUpdateDto},
        service::SensorService,
    },
    utils::{errors::ServiceErr, validation::ValidatedJson},
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
        request_body = SensorDto,
        responses(
            (status = 201, description = "Sensor created"),
        )
    )]
pub async fn post_sensors(
    State(srv): State<Arc<SensorService>>,
    ValidatedJson(dto): ValidatedJson<SensorDto>,
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
pub async fn get_sensor(State(srv): State<Arc<SensorService>>, Path(name): Path<String>) -> impl IntoResponse {
    if !name.validate_length(Some(1), Some(200), None) {
        tracing::debug!("Name validation failed for get_sensor with value: {}", name);

        return ServiceErr::Validation("Name must be between 1 and 200 chars".into()).into_response();
    }

    srv.get(&name).await.map(Json).into_response()
}

#[utoipa::path(
        put,
        path = "/sensors/{name}",
        request_body = SensorUpdateDto,
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
    ValidatedJson(dto): ValidatedJson<SensorUpdateDto>,
) -> impl IntoResponse {
    srv.update(&name, &dto).await.map(|_| StatusCode::OK)
}

#[utoipa::path(
        post,
        path = "/sensors/nearby",
        request_body = SensorNearbyDto,
        responses(
            (status = 200, description = "Nearby Sensor found"),
        )
    )]
pub async fn post_nearby(
    State(srv): State<Arc<SensorService>>,
    ValidatedJson(dto): ValidatedJson<SensorNearbyDto>,
) -> impl IntoResponse {
    srv.nearby(dto.lat, dto.long).await.map(Json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, Response},
    };
    use rstest::rstest;
    use tower::ServiceExt;

    #[rstest]
    #[case(&"a".repeat(201), false, 400, "Name")]
    #[case(&"".repeat(201), false, 404, "")] // 404, since name param can't be picked from empty
    #[case(&" ".repeat(1), true, 0, "")] // http::Error(InvalidUri(InvalidUriChar))
    #[case(&" ".repeat(1_000_000), true, 0, "")] // http::Error(InvalidUri(TooLong))
    #[case(&"a".repeat(1_000_000), true, 0, "")] // http::Error(InvalidUri(TooLong))
    #[tokio::test]
    async fn test_get_sensor(#[case] name: &str, #[case] invalid_req: bool, #[case] status: u16, #[case] field: &str) {
        let req = Request::get(format!("/sensors/{name}")).body(Body::empty());

        if invalid_req {
            assert_eq!(req.is_err(), true);
            return;
        }

        let response = sensor_api(PgPool::connect_lazy("postgres://mock:mock@localhost/mock").unwrap())
            .oneshot(req.unwrap())
            .await
            .unwrap();

        assert_err(response, status, field).await;
    }

    #[rstest]
    #[case(&"a".repeat(201), 0.0, 0.0, "name")]
    #[case("", 0.0, 0.0, "name")]
    #[case("a", f64::MAX, 0.0, "lat")]
    #[case("a", -90.1, 0.0, "lat")]
    #[case("a", 0.0, f64::MAX, "long")]
    #[case("a", 0.0, -180.1, "long")]
    #[tokio::test]
    async fn test_post_sensors(#[case] name: &str, #[case] lat: f64, #[case] long: f64, #[case] field: &str) {
        let dto = SensorDto {
            name: name.to_string(),
            lat,
            long,
            tags: Default::default(),
        };
        let req = Request::post("/sensors")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&dto).unwrap()));

        let response = sensor_api(PgPool::connect_lazy("postgres://mock:mock@localhost/mock").unwrap())
            .oneshot(req.unwrap())
            .await
            .unwrap();

        assert_err(response, 400, field).await;
    }

    #[rstest]
    #[case(f64::MAX, 0.0, "lat")]
    #[case(-90.1, 0.0, "lat")]
    #[case(0.0, f64::MAX, "long")]
    #[case(0.0, -180.1, "long")]
    #[tokio::test]
    async fn test_put_sensor(#[case] lat: f64, #[case] long: f64, #[case] field: &str) {
        let body = serde_json::json!({ "lat": lat, "long": long, "tags": {"a": "1"} });
        let req = Request::put("/sensors/a")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()));

        let response = sensor_api(PgPool::connect_lazy("postgres://mock:mock@localhost/mock").unwrap())
            .oneshot(req.unwrap())
            .await
            .unwrap();

        assert_err(response, 400, field).await;
    }

    #[rstest]
    #[case(f64::MAX, 0.0, "lat")]
    #[case(-90.1, 0.0, "lat")]
    #[case(0.0, f64::MAX, "long")]
    #[case(0.0, -180.1, "long")]
    #[tokio::test]
    async fn test_post_nearby(#[case] lat: f64, #[case] long: f64, #[case] field: &str) {
        let body = serde_json::json!({ "lat": lat, "long": long });
        let req = Request::post("/sensors/nearby")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()));

        let response = sensor_api(PgPool::connect_lazy("postgres://mock:mock@localhost/mock").unwrap())
            .oneshot(req.unwrap())
            .await
            .unwrap();

        assert_err(response, 400, field).await;
    }

    async fn assert_err(res: Response<Body>, status: u16, field: &str) {
        assert_eq!(res.status().as_u16(), status);

        if field.is_empty() {
            return;
        }

        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["detail"].as_str().unwrap().contains(field), "{json}");
    }
}
