use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

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

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct SensorDto {
    #[validate(length(min = 1, max = 200, message = "Must be between 1 and 200"))]
    pub name: String,

    #[validate(range(min = -90.0, max = 90.0, message = "Must be between -90deg and 90deg"))]
    pub lat: f64,

    #[validate(range(min = -180.0, max = 180.0, message = "Must be between -180deg and 180deg"))]
    pub long: f64,

    #[validate(length(min = 0, max = 100))]
    #[validate(custom(function = "validate_tags"))]
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

#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct SensorUpdateDto {
    #[validate(range(min = -90.0, max = 90.0, message = "Must be between -90deg and 90deg"))]
    pub lat: f64,

    #[validate(range(min = -180.0, max = 180.0, message = "Must be between -180deg and 180deg"))]
    pub long: f64,

    #[validate(length(min = 0, max = 100))]
    #[validate(custom(function = "validate_tags"))]
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct SensorNearbyDto {
    #[validate(range(min = -90.0, max = 90.0, message = "Must be between -90deg and 90deg"))]
    pub lat: f64,

    #[validate(range(min = -180.0, max = 180.0, message = "Must be between -180deg and 180deg"))]
    pub long: f64,
}

fn validate_tags(tags: &BTreeMap<String, String>) -> Result<(), ValidationError> {
    let err = ValidationError::new("tag_validation");

    if tags.keys().any(|k| k.trim().is_empty()) {
        return Err(err.with_message("Keys of Tags must not be empty".into()));
    }
    if tags.keys().any(|k| k.len() > 200) {
        return Err(err.with_message("Keys of Tags must not be longer than 200".into()));
    }

    if tags.values().any(|k| k.trim().is_empty()) {
        return Err(err.with_message("Values of Tags must not be empty".into()));
    }
    if tags.values().any(|k| k.len() > 200) {
        return Err(err.with_message("Values of Tags must not be longer than 200".into()));
    }
    Ok(())
}
