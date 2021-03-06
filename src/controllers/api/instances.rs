use actix_web::web::Json;
use paperclip::actix::{api_v2_operation, get, post, web, Apiv2Schema, CreatedJson};
use paperclip_actix::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::db;
use crate::model::instance::{CreateInstanceBody, InstanceRow, InstanceStatus};
use crate::model::{AuthAppResult, CreatedAuthAppResult};

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct ClientIdPathInfo {
    client_id: String,
}

#[api_v2_operation]
#[get("/status/{client_id}")]
async fn get_instance_status(
    conn: web::Data<Pool<Postgres>>,
    clientid_path: web::Path<ClientIdPathInfo>,
) -> AuthAppResult<InstanceStatus> {
    db::instance_status::get_instance_status(conn, clientid_path.client_id.clone())
        .await
        .map(Json)
}

#[api_v2_operation]
#[post("/extend/{client_id}")]
async fn extend_trial(
    conn: web::Data<Pool<Postgres>>,
    clientid_path: web::Path<ClientIdPathInfo>,
) -> AuthAppResult<InstanceStatus> {
    db::instance_status::extend_trial(conn, clientid_path.client_id.clone())
        .await
        .map(Json)
}

#[api_v2_operation]
async fn create_instance(
    conn: web::Data<Pool<Postgres>>,
    body: Json<CreateInstanceBody>,
) -> CreatedAuthAppResult<InstanceRow> {
    db::instance::create(conn.as_ref(), body.into_inner())
        .await
        .map(CreatedJson)
}

#[api_v2_operation]
async fn list_instances(conn: web::Data<Pool<Postgres>>) -> AuthAppResult<Vec<InstanceRow>> {
    db::instance::list_all(conn.as_ref()).await.map(Json)
}

pub fn configure_instances(cfg: &mut ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::get().to(list_instances))
            .route(web::post().to(create_instance)),
    )
    .service(get_instance_status)
    .service(extend_trial);
}
