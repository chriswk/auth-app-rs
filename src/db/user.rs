use crate::errors::AuthAppError;
use crate::model::instance::{CreateInstanceBody, InstanceRow, InstanceState};
use actix_web::web::Data;
use paperclip_actix::web;
use sqlx::{Pool, Postgres};

pub async fn create(
    conn: web::Data<Pool<Postgres>>,
    create_request: CreateInstanceBody,
) -> Result<InstanceRow, AuthAppError> {
    let client_id = create_request.client_id.clone();
    let display_name = create_request.display_name.clone();
    let email_domain = create_request.email_domain.clone();
    let plan = create_request.plan.clone();
    let instance_state = InstanceState::Unassigned.to_string();
    let region = create_request.region.clone();
    let seats = 5;
    let trial_extended = 0;
    let billing_center = create_request.billing_center.clone();
    sqlx::query_as!(InstanceRow, r#"
        INSERT INTO 
            instances(client_id, display_name, email_domain, instance_state, plan, region, seats, billing_center, trial_extended)
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
        RETURNING *;
    "#, client_id, display_name, email_domain, instance_state, plan, region, seats, billing_center, trial_extended)
        .fetch_one(conn.as_ref())
        .await
        .map_err(AuthAppError::SqlError)
}

pub(crate) async fn list_all(conn: Data<Pool<Postgres>>) -> Result<Vec<InstanceRow>, AuthAppError> {
    sqlx::query_as!(
        InstanceRow,
        r#"
        SELECT * FROM instances;
    "#
    )
    .fetch_all(conn.as_ref())
    .await
    .map_err(AuthAppError::SqlError)
}
