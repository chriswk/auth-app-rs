use std::iter;

use actix_web::{delete, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::errors::AuthAppError;

#[derive(Deserialize)]
struct Client {
    pub client_id: String,
}

pub fn default_role() -> String {
    return "writer".to_string();
}

#[derive(Deserialize, Serialize)]
pub struct NewUserBody {
    pub email: String,
    #[serde(default = "default_role")]
    pub role: String,
}

#[derive(Deserialize, Serialize)]
struct DeleteUserBody {
    email: String,
}

#[derive(Deserialize, Serialize)]
struct EmailsBody {
    emails: Vec<String>,
}

#[post("/{client_id}/add")]
async fn add_user(
    client: web::Path<Client>,
    new_user_body: web::Json<NewUserBody>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, AuthAppError> {
    let client_id = client.client_id.clone();
    let email = new_user_body.email.clone();
    let role = new_user_body.role.clone();
    sqlx::query!(
        r#"
        INSERT INTO user_access(client_id, email, role) VALUES ($1, $2, $3)
    "#,
        client_id,
        email,
        role
    )
    .execute(pool.as_ref())
    .await
    .map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Created().finish())
}

#[delete("/{client_id}/remove")]
async fn remove_user(
    client: web::Path<Client>,
    delete_user_body: web::Json<DeleteUserBody>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, AuthAppError> {
    let client_id = client.client_id.clone();
    let email = delete_user_body.email.clone();
    sqlx::query!(
        r#"
            DELETE FROM user_access WHERE client_id = $1 AND email = $2
        "#,
        client_id,
        email
    )
    .execute(pool.as_ref())
    .await
    .map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/{clientId}/sync")]
async fn sync_users(
    client: web::Path<Client>,
    emails_body: web::Json<EmailsBody>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, AuthAppError> {
    let client_id: String = client.client_id.clone();
    let emails = emails_body.emails.clone();
    let mut tx = pool.as_ref().begin().await?;
    sqlx::query!(
        r#"
            DELETE FROM user_access WHERE client_id = $1
        "#,
        client_id
    )
    .execute(&mut tx)
    .await
    .map_err(AuthAppError::SqlError)?;

    let role_repeat: Vec<String> = iter::repeat("admin".to_string())
        .take(emails.len())
        .collect();
    let client_id_repeat: Vec<String> = iter::repeat(client_id).take(emails.len()).collect();
    sqlx::query(
        r#"
            INSERT INTO user_access(client_id, email, role)
            SELECT client_id, email, role
            FROM UNNEST($1, $2, $3)
        "#,
    )
    .bind(&client_id_repeat)
    .bind(&emails)
    .bind(&role_repeat)
    .execute(&mut tx)
    .await
    .map_err(AuthAppError::SqlError)?;
    tx.commit().await.map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Accepted().finish())
}

pub fn configure_instance_user_service(cfg: &mut web::ServiceConfig) {
    cfg.service(add_user)
        .service(remove_user)
        .service(sync_users);
}
