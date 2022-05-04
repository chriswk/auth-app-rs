use crate::errors::AuthAppError;
use actix_web::{delete, post, web, HttpResponse};
use sqlx::{Pool, Postgres};

#[post("/:clientId/add")]
async fn add_user(pool: web::Data<Pool<Postgres>>) -> Result<HttpResponse, AuthAppError> {
    Ok(HttpResponse::Created().finish())
}

#[delete("/:clientId/remove")]
async fn remove_user(pool: web::Data<Pool<Postgres>>) -> Result<HttpResponse, AuthAppError> {
    Ok(HttpResponse::Ok().finish())
}

#[post("/:clientId/sync")]
async fn sync_users(pool: web::Data<Pool<Postgres>>) -> Result<HttpResponse, AuthAppError> {
    Ok(HttpResponse::Accepted().finish())
}

pub fn configure_instance_user_service(cfg: &mut web::ServiceConfig) {
    cfg.service(add_user)
        .service(remove_user)
        .service(sync_users);
}
