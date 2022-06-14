use actix_web::web::Json;
use paperclip::actix::{api_v2_operation, get, post, put, web, Apiv2Schema};
use paperclip_actix::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::db;
use crate::model::instance::InstanceStatus;

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct ClientIdPathInfo {
    client_id: String,
}

#[api_v2_operation]
#[get("/{client_id}/status")]
async fn get_instance_status(
    conn: web::Data<Pool<Postgres>>,
    clientid_path: web::Path<ClientIdPathInfo>,
) -> Json<InstanceStatus> {
    db::instance_status::get_instance_status(conn, clientid_path.client_id.clone())
        .await
        .map(|s| Json(s))
        .unwrap()
}

#[api_v2_operation]
#[post("/{client_id}/extend")]
async fn extend_trial(
    conn: web::Data<Pool<Postgres>>,
    clientid_path: web::Path<ClientIdPathInfo>,
) -> Json<InstanceStatus> {
    db::instance_status::extend_trial(conn, clientid_path.client_id.clone())
        .await
        .map(|s| Json(s))
        .unwrap()
}

pub fn configure_instances(cfg: &mut ServiceConfig) {
    cfg.service(get_instance_status).service(extend_trial);
}
