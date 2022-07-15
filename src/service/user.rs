use crate::db;
use crate::errors::AuthAppError;
use crate::model::user::{MinimalAuthUser, Role};
use sqlx::{Pool, Postgres};

pub async fn get_or_create_user(
    conn: &Pool<Postgres>,
    email: String,
) -> Result<MinimalAuthUser, AuthAppError> {
    let user_exists = db::user::user_exists(conn, email.clone()).await?;
    if !user_exists {
        let domain = email.split("@").last().unwrap();
        match db::instance::get_instance_for_domain(conn, domain.to_string()).await {
            Ok(instance) => {
                db::user_access::add_access(conn, instance.client_id, email.clone(), Role::WRITE)
                    .await?;
                db::user::get_user(conn, email.clone()).await
            }
            Err(e) => Err(AuthAppError::DomainNotAllowed),
        }
    } else {
        db::user::get_user(conn, email.clone()).await
    }
}
