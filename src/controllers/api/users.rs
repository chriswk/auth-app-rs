use actix_web::web::Json;
use paperclip::actix::{api_v2_operation, web, Apiv2Schema, CreatedJson};
use paperclip_actix::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::db;
use crate::model::user::{CreateUserBody, MinimalAuthUser};
use crate::model::CreatedAuthAppResult;

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct ClientIdPathInfo {
    client_id: String,
}

#[api_v2_operation]
async fn create_user(
    conn: web::Data<Pool<Postgres>>,
    body: Json<CreateUserBody>,
) -> CreatedAuthAppResult<MinimalAuthUser> {
    db::user::create(conn, body.into_inner())
        .await
        .map(CreatedJson)
}

pub fn configure_users(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(create_user)));
}
