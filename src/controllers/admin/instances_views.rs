use crate::db;
use crate::errors::AuthAppError;
use crate::model::instance::InstanceRow;
use actix_web::HttpResponse;
use handlebars::Handlebars;
use paperclip::actix::HttpResponseWrapper;
use paperclip_actix::web;
use paperclip_actix::web::ServiceConfig;
use serde::Serialize;
use sqlx::{Pool, Postgres};

#[derive(Serialize)]
pub struct InstanceTemplateData {
    instances: Vec<InstanceRow>,
}

pub async fn list_instances(
    conn: web::Data<Pool<Postgres>>,
    hb: web::Data<Handlebars<'_>>,
) -> HttpResponseWrapper {
    let instances = db::instance::list_all(conn.as_ref()).await.unwrap();
    let data = InstanceTemplateData { instances };

    hb.as_ref()
        .render::<InstanceTemplateData>("admin/instances/list", &data)
        .map(|html| HttpResponse::Ok().body(html))
        .map(HttpResponseWrapper)
        .unwrap_or_else(|_| HttpResponseWrapper(HttpResponse::InternalServerError().finish()))
}

pub fn configure_admin_instance_views(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_instances)));
}
