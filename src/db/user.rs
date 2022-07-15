use crate::errors::AuthAppError;
use crate::model::user::{CreateUserBody, DeleteUserRequest, MinimalAuthUser};
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

fn generate_passwords(number_to_generate: usize) -> Vec<String> {
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
    gen.generate(number_to_generate).unwrap()
}

pub async fn create_user(
    conn: &Pool<Postgres>,
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
    .fetch_one(conn)
    .await
    .map_err(AuthAppError::SqlError)
}

pub async fn user_exists(conn: &Pool<Postgres>, email: String) -> Result<bool, AuthAppError> {
    sqlx::query_as!(
        crate::model::Exists,
        r#"
        SELECT EXISTS (SELECT 1 FROM auth_users WHERE email = $1) AS exists
    "#,
        email
    )
    .fetch_one(conn)
    .await
    .map(|e| e.exists.unwrap_or(false))
    .map_err(AuthAppError::SqlError)
}

pub async fn user_access_exists(
    conn: &Pool<Postgres>,
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
    .fetch_one(conn)
    .await
    .map(|e| e.exists.unwrap_or(false))
    .map_err(AuthAppError::SqlError)
}

pub async fn create(
    conn: &Pool<Postgres>,
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
            .execute(conn)
            .await
            .map_err(AuthAppError::SqlError)?;
            Ok(user)
        }
    }
}

pub async fn delete(
    conn: &Pool<Postgres>,
    delete_request: DeleteUserRequest,
) -> Result<(), AuthAppError> {
    sqlx::query!(
        "DELETE FROM user_access WHERE client_id = $1 AND email = $2",
        delete_request.client_id.clone(),
        delete_request.email.clone()
    )
    .execute(conn)
    .await
    .map_err(AuthAppError::SqlError)
    .map(|_| ())
}

pub async fn get_user(
    conn: &Pool<Postgres>,
    email: String,
) -> Result<MinimalAuthUser, AuthAppError> {
    sqlx::query_as!(
        MinimalAuthUser,
        "SELECT email, name FROM auth_users WHERE email = $1",
        email
    )
    .fetch_one(conn)
    .await
    .map_err(AuthAppError::SqlError)
}

pub async fn sync_users(
    conn: &Pool<Postgres>,
    client_id: String,
    emails: Vec<String>,
) -> Result<(), AuthAppError> {
    let client_ids = vec![client_id; emails.len()];
    let passwords = generate_passwords(emails.len());
    let mut tx = conn.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO auth_users(email, password_hash)
            SELECT client_id, password_hash FROM UNNEST($1, $2) AS a(client_id, password_hash)
            ON CONFLICT DO NOTHING;
    "#,
    )
    .bind(&client_ids)
    .bind(&passwords)
    .execute(&mut tx)
    .await
    .map_err(AuthAppError::SqlError)?;
    sqlx::query(
        r#"
        INSERT INTO user_access(client_id, email, role)
        SELECT client_id, email, 'writer' FROM
        UNNEST($1, $2) AS a(client_id, email)"#,
    )
    .bind(&client_ids)
    .bind(&emails)
    .execute(&mut tx)
    .await
    .map_err(|e| AuthAppError::SqlError(e))
    .map(|_| ())
}
