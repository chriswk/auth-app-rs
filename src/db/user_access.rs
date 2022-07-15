use crate::errors::AuthAppError;
use crate::model::user::Role;
use sqlx::{Pool, Postgres};

pub async fn add_access(
    conn: &Pool<Postgres>,
    client_id: String,
    email: String,
    role: Role,
) -> Result<(), AuthAppError> {
    sqlx::query!(
        r#"
        INSERT INTO user_access(client_id, email, role) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (client_id, email) DO NOTHING;
    "#,
        client_id,
        email,
        role.to_string()
    )
    .execute(conn)
    .await
    .map_err(AuthAppError::SqlError)
    .map(|_| ())
}
