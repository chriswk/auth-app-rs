use crate::errors::AuthAppError::SqlError;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, From};

#[derive(Display, From, Debug)]
pub enum AuthAppError {
    SqlError(sqlx::Error),
}
impl std::error::Error for AuthAppError {}

const UNIQUE_VIOLATION: &str = "23505";

impl ResponseError for AuthAppError {
    fn status_code(&self) -> StatusCode {
        match self {
            SqlError(e) => match e {
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                sqlx::Error::PoolTimedOut => StatusCode::REQUEST_TIMEOUT,
                sqlx::Error::Decode(_) => StatusCode::BAD_REQUEST,
                sqlx::Error::Database(e) => match e.code() {
                    Some(c) => match c.to_lowercase().as_str() {
                        UNIQUE_VIOLATION => StatusCode::CONFLICT,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    },
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).finish()
    }
}
