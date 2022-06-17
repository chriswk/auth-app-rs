use crate::errors::AuthAppError;
use crate::model::user::{CreateUserBody, MinimalAuthUser};
use actix_web::web::Data;
use paperclip_actix::web;
use passwords::PasswordGenerator;
use sqlx::{Pool, Postgres};

fn generate_password() -> String {
    let gen = PasswordGenerator {
        length: 32,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        exclude_similar_characters: true,
        strict: true,
        spaces: false,
    };
    gen.generate_one().unwrap()
}

pub async fn create_user(
    conn: web::Data<Pool<Postgres>>,
    email: String,
) -> Result<MinimalAuthUser, AuthAppError> {
    sqlx::query_as!(
        MinimalAuthUser,
        r#"
        INSERT INTO auth_users(email, password_hash) VALUES ($1, $2) ON CONFLICT(email) DO NOTHING RETURNING email, name 
    "#,
        email,
        generate_password()
    )
    .fetch_one(conn.as_ref())
    .await
    .map_err(AuthAppError::SqlError)
}

pub async fn user_access_exists(
    conn: web::Data<Pool<Postgres>>,
    client_id: String,
    email: String,
) -> Result<bool, AuthAppError> {
    sqlx::query_as!(
        crate::model::Exists,
        r#"
        SELECT EXISTS (SELECT 1 FROM user_access WHERE client_id = $1 AND email = $2) AS exists;
    "#,
        client_id,
        email
    )
    .fetch_one(conn.as_ref())
    .await
    .map(|e| e.exists.unwrap_or(false))
    .map_err(AuthAppError::SqlError)
}

pub async fn create(
    conn: Data<Pool<Postgres>>,
    create_request: CreateUserBody,
) -> Result<MinimalAuthUser, AuthAppError> {
    let user_already_has_access: bool = user_access_exists(
        conn,
        create_request.client_id.clone(),
        create_request.email.clone(),
    )
    .await?;
    match user_already_has_access {
        true => Err(AuthAppError::UserAlreadyHasAccess),
        false => {
            let user = create_user(conn, create_request.email.clone()).await?;
            let client_id = create_request.client_id.clone();
            let email = create_request.email.clone();
            let role = create_request.role.clone();
            sqlx::query(
                r#"
                INSERT INTO user_access(client_id, email, role) VALUES ($1, $2, $3);
            "#,
            )
            .bind(client_id)
            .bind(email)
            .bind(role)
            .execute(conn.as_ref())
            .await
            .map_err(AuthAppError::SqlError)?;
            Ok(user)
        }
    }
}
