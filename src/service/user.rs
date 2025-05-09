use crate::db;
use crate::errors::AuthAppError;
use crate::model::user::{MinimalAuthUser, Role};
use log::{info, warn};
use sqlx::{Pool, Postgres};

pub async fn get_or_create_user(
    conn: &Pool<Postgres>,
    email: String,
) -> Result<MinimalAuthUser, AuthAppError> {
    let user_exists = db::user::user_exists(conn, &email).await?;
    if !user_exists {
        let domain = email.split('@').last().unwrap();
        warn!("Getting instances for {:#?}", domain);
        match db::instance::get_instance_for_domain(conn, domain.to_string()).await {
            Ok(instance) => {
                warn!("Instance was fine: {:#?}", instance);
                let user = db::user::create_user(conn, &email).await?;
                db::user_access::add_access(conn, &instance.client_id, &email, Role::WRITE)
                    .await?;
                Ok(user)
            }
            Err(_) => {
                info!("Something was wrong with instance for domain {domain}");
                Err(AuthAppError::DomainNotAllowed)
            }
        }
    } else {
        db::user::get_user(conn, &email).await
    }
}
