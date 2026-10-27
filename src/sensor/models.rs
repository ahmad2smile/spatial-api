use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Sensor {
    pub name: String,
    lat: f64,
    long: f64,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Tag {
    sensor: String,
    name: String,
    value: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SensorDto {
    pub name: String,
    pub lat: f64,
    pub long: f64,
    pub tags: BTreeMap<String, String>,
}

impl SensorDto {
    pub fn from(sensor: Sensor, tags: Vec<Tag>) -> Self {
        Self {
            name: sensor.name,
            lat: sensor.lat,
            long: sensor.long,
            tags: tags.into_iter().map(|t| (t.name, t.value)).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SensorUpdateDto {
    pub lat: f64,
    pub long: f64,
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SensorNearbyDto {
    pub lat: f64,
    pub long: f64,
}
