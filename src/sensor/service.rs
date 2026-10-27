use std::collections::BTreeMap;

use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use sqlx::{PgPool, Postgres};

use crate::sensor::models::{Sensor, SensorDto, SensorUpdateDto, Tag};
use crate::utils::errors::ServiceErr;
use crate::utils::errors::ServiceResult;

#[derive(Clone)]
pub struct SensorService {
    pool: sqlx::PgPool,
}

impl SensorService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn create_tags(
        &self,
        name: &str,
        tags: &BTreeMap<String, String>,
    ) -> Query<'_, Postgres, PgArguments> {
        sqlx::query(
            "INSERT INTO tags (sensor, name, value)
            SELECT $1, n, v FROM UNNEST($2::text[], $3::text[]) AS t(n, v)",
        )
        .bind(name)
        .bind(tags.keys().cloned().collect::<Vec<_>>())
        .bind(tags.values().cloned().collect::<Vec<_>>())
    }

    pub async fn create(&self, dto: &SensorDto) -> ServiceResult<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT INTO sensors (name, lat, long) VALUES ($1, $2, $3)")
            .bind(&dto.name)
            .bind(&dto.lat)
            .bind(&dto.long)
            .execute(&mut *tx)
            .await?;

        self.create_tags(&dto.name, &dto.tags)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get(&self, name: &str) -> ServiceResult<SensorDto> {
        let sensor: Sensor = sqlx::query_as("SELECT * FROM sensors WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| ServiceErr::NotFound(format!("Sensor {name}")))?;

        let tags: Vec<Tag> = sqlx::query_as("SELECT * FROM tags WHERE sensor = $1")
            .bind(name)
            .fetch_all(&self.pool)
            .await?;

        Ok(SensorDto::from(sensor, tags))
    }

    pub async fn update(&self, name: &str, dto: &SensorUpdateDto) -> ServiceResult<()> {
        let mut tx = self.pool.begin().await?;

        let res = sqlx::query(
            "UPDATE sensors SET lat = COALESCE($1, lat), long = COALESCE($2, long) WHERE name = $3",
        )
        .bind(dto.lat)
        .bind(dto.long)
        .bind(name)
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() == 0 {
            return Err(ServiceErr::NotFound(format!("Sensor {name}")));
        }

        if !dto.tags.is_empty() {
            sqlx::query("DELETE FROM tags WHERE sensor = $1")
                .bind(&name)
                .execute(&mut *tx)
                .await?;

            self.create_tags(&name, &dto.tags).execute(&mut *tx).await?;
        }

        Ok(())
    }

    pub async fn nearby(&self, lat: f64, long: f64) -> ServiceResult<Sensor> {
        let sensor: Sensor = sqlx::query_as(
            "SELECT * FROM sensors
            ORDER BY position <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography LIMIT 1",
        )
        .bind(lat)
        .bind(long)
        .fetch_one(&self.pool)
        .await?;

        Ok(sensor)
    }
}
