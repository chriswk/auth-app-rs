use crate::model::health::Health;
use paperclip::actix::web::Json;
use paperclip::actix::{api_v2_operation, get};

#[api_v2_operation]
#[get("/healthz")]
pub async fn healthy() -> Json<Health> {
    let health = Health {
        status: "OK".to_string(),
    };
    Json(health)
}
