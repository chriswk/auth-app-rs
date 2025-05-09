use actix_web::web::{Data, Json, Path};
use log::warn;
use paperclip::actix::{api_v2_operation, web, Apiv2Schema, CreatedJson};
use paperclip_actix::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::db;
use crate::model::user::{
    CreateUserBody, DeleteUserBody, DeleteUserRequest, MinimalAuthUser, SyncUserBody,
};
use crate::model::{AuthAppResult, CreatedAuthAppResult};

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct ClientIdPathInfo {
    client_id: String,
}

#[api_v2_operation]
async fn create_user(
    conn: Data<Pool<Postgres>>,
    body: Json<CreateUserBody>,
) -> CreatedAuthAppResult<MinimalAuthUser> {
    db::user::create(conn.as_ref(), body.into_inner())
        .await
        .map(CreatedJson)
}

#[api_v2_operation]
async fn remove_user(
    conn: Data<Pool<Postgres>>,
    client_id: Path<ClientIdPathInfo>,
    body: Json<DeleteUserBody>,
) -> AuthAppResult<()> {
    db::user::delete(
        conn.as_ref(),
        DeleteUserRequest {
            client_id: client_id.client_id.clone(),
            email: body.email.clone(),
        },
    )
        .await
        .map(Json)
}

#[api_v2_operation]
async fn sync_users(
    conn: Data<Pool<Postgres>>,
    client_id: Path<ClientIdPathInfo>,
    body: Json<SyncUserBody>,
) -> AuthAppResult<()> {
    warn!("Entered sync operation");
    db::user::sync_users(
        conn.as_ref(),
        &client_id.client_id,
        body.emails.clone(),
    )
        .await
        .map(Json)
}

pub fn configure_users(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{client_id}")
            .service(web::resource("/create").route(web::post().to(create_user)))
            .service(web::resource("/remove").route(web::delete().to(remove_user)))
            .service(web::resource("/sync").route(web::post().to(sync_users))),
    );
}
