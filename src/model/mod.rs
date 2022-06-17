use crate::errors::AuthAppError;
use actix_web::web::Json;
use paperclip::actix::CreatedJson;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub mod health;
pub mod instance;
pub mod user;
pub mod version_info;

pub type AuthAppResult<T> = Result<Json<T>, AuthAppError>;
pub type CreatedAuthAppResult<T> = Result<CreatedJson<T>, AuthAppError>;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Exists {
    pub exists: Option<bool>,
}
