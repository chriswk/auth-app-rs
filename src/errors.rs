use crate::errors::AuthAppError::SqlError;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, From};
use log::warn;
use paperclip::actix::api_v2_errors;

#[derive(Display, From, Debug)]
#[api_v2_errors(
    400,
    description = "Sorry, bad request",
    code = 401,
    description = "Not Authorized"
    code = 403,
    description = "Forbidden, go away",
    code = 404,
    description = "Not found",
    code = 408,
    description = "Request timeout",
    code = 409,
    description = "This id already exists",
    500,
    description = "Internal Server Error",
)]
pub enum AuthAppError {
    UserAlreadyHasAccess,
    SqlError(sqlx::Error),
    DomainNotAllowed,
    AccessNotAllowed,
    InvalidToken,
    InvalidCookieDuration,
    TokenExpired,
    RenderError,
}
impl std::error::Error for AuthAppError {}

const UNIQUE_VIOLATION: &str = "23505";
const FOREIGN_KEY_VIOLATION: &str = "23503";

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
                        FOREIGN_KEY_VIOLATION => StatusCode::BAD_REQUEST,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    },
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AuthAppError::UserAlreadyHasAccess => StatusCode::CONFLICT,
            AuthAppError::DomainNotAllowed => StatusCode::UNAUTHORIZED,
            AuthAppError::AccessNotAllowed => StatusCode::UNAUTHORIZED,
            AuthAppError::InvalidToken => StatusCode::FORBIDDEN,
            AuthAppError::InvalidCookieDuration => StatusCode::BAD_REQUEST,
            AuthAppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthAppError::RenderError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        warn!("status code for error {}", self.status_code());
        HttpResponse::build(self.status_code()).finish()
    }
}
