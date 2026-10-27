use axum::{http::StatusCode, response::IntoResponse};
use sqlx::{error::ErrorKind, postgres::PgDatabaseError};
use utoipa::ToSchema;

use crate::utils::errors::ServiceErr::{Conflict, Database, NotFound, Validation};

#[derive(Debug, thiserror::Error, ToSchema)]
pub enum ServiceErr {
    #[error("Entity not found with value: {0}.")]
    NotFound(String),

    #[error("Invalid value: {0}.")]
    Validation(String),

    #[error("Already exists: {0}.")]
    Conflict(String),

    #[error("Something went wrong with data handling, please try again.")]
    #[schema(value_type = String)]
    Database(#[source] sqlx::Error),
}

pub type ServiceResult<T> = Result<T, ServiceErr>;

impl From<sqlx::Error> for ServiceErr {
    fn from(error: sqlx::Error) -> Self {
        if let Some(db_err) = error.as_database_error() {
            let column = db_err
                .try_downcast_ref::<PgDatabaseError>()
                .and_then(|e| e.column())
                .unwrap_or("value");

            match db_err.kind() {
                ErrorKind::UniqueViolation => return Conflict("Record already exists".into()),
                ErrorKind::ForeignKeyViolation => {
                    return NotFound("Referenced record not found".into());
                }
                ErrorKind::NotNullViolation => {
                    return Validation(format!("Missing required field: {column}"));
                }
                ErrorKind::CheckViolation | ErrorKind::ExclusionViolation => {
                    return Validation(format!("Invalid {column}"));
                }
                _ => {}
            }
        }

        match error {
            sqlx::Error::InvalidArgument(msg) => Validation(msg),
            sqlx::Error::RowNotFound => NotFound("No matching record".into()),
            other => Database(other),
        }
    }
}

impl IntoResponse for ServiceErr {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            NotFound(_) => StatusCode::NOT_FOUND,
            Validation(_) => StatusCode::BAD_REQUEST,
            Conflict(_) => StatusCode::CONFLICT,
            Database(err) => {
                tracing::error!(error = ?err, "Database error");

                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let msg = self.to_string();

        if status.is_client_error() {
            tracing::debug!(error = %msg, status = %status, "Client error");
        }

        (status, msg).into_response()
    }
}
