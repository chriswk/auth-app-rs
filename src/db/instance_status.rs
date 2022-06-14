use crate::errors::AuthAppError;
use crate::model::instance::InstanceStatus;
use paperclip_actix::web;
use sqlx::{Pool, Postgres};

pub async fn get_instance_status(
    conn: web::Data<Pool<Postgres>>,
    client_id: String,
) -> Result<InstanceStatus, AuthAppError> {
    sqlx::query_as!(
        InstanceStatus,
        r#"
        SELECT 
         plan,
         trial_expiry, 
         trial_start, 
         trial_extended,
         instance_state, 
         billing_center, 
         region 
         FROM instances 
         WHERE client_id = $1;
    "#,
        client_id
    )
    .fetch_one(conn.as_ref())
    .await
    .map_err(AuthAppError::SqlError)
}

pub async fn extend_trial(
    conn: web::Data<Pool<Postgres>>,
    client_id: String,
) -> Result<InstanceStatus, AuthAppError> {
    sqlx::query_as!(
        InstanceStatus,
        r#"
        UPDATE instances SET trial_extended = trial_extended + 1,
                trial_expiry = trial_expiry + INTERVAL '5 DAYS'
                 WHERE client_id = $1 AND instance_state = 'Trial'
        RETURNING plan, trial_expiry, trial_start, trial_extended, instance_state, billing_center, region 
    "#, client_id
    ).fetch_one(conn.as_ref())
        .await.map_err(AuthAppError::SqlError)
}
