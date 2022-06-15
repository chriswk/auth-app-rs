use crate::errors::AuthAppError;
use actix_web::web::Json;
use paperclip::actix::CreatedJson;

pub mod health;
pub mod instance;
pub mod version_info;

pub type AuthAppResult<T> = Result<Json<T>, AuthAppError>;
pub type CreatedAuthAppResult<T> = Result<CreatedJson<T>, AuthAppError>;
