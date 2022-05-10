use std::borrow::Borrow;
use std::string::ToString;

use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Pool, Postgres};
use strum::IntoStaticStr;
use strum_macros::Display;

use crate::errors::AuthAppError;

#[derive(Serialize, Deserialize)]
pub struct InstanceInfo {
    client_id: String,
    plan: String,
    display_name: Option<String>,
    stripe_customer_id: Option<String>,
    email_domain: Option<String>,
    region: String,
    created_at: DateTime<Utc>,
    trial_expiry: Option<DateTime<Utc>>,
    trial_start: Option<DateTime<Utc>>,
    instance_state: String,
    trial_extended: i32,
    billing_center: String,
    seats: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PathInfo {
    client_id: String,
}

fn default_plan() -> String {
    "pro".to_string()
}

fn default_region() -> String {
    "eu".to_string()
}

#[derive(Serialize, Deserialize, Default)]
pub struct NewInstanceBody {
    pub client_id: String,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_plan")]
    pub plan: String,
    pub display_name: Option<String>,
    pub email_domain: Option<String>,
    pub billing_center: Option<String>,
}

#[derive(Serialize, Deserialize, IntoStaticStr, Display, Clone)]
pub enum InstanceState {
    Unassigned,
    Trial,
    Active,
    TrialExpired,
    Churned,
}

pub fn default_instance_state() -> InstanceState {
    InstanceState::Unassigned
}

#[derive(Serialize, Deserialize)]
pub struct AssignInstance {
    pub admin_email: String,
    pub display_name: String,
    pub instance_state: Option<InstanceState>,
    pub email_domain: Option<String>,
}

pub async fn create_instance_db(
    new_instance: NewInstanceBody,
    conn: &Pool<Postgres>,
) -> Result<PgQueryResult, AuthAppError> {
    let client_id = new_instance.client_id.clone();
    let plan = new_instance.plan.clone();
    let display_name = new_instance.display_name.clone();
    let email_domain = new_instance.email_domain.clone();
    let region = new_instance.region.clone();
    let billing_center = new_instance
        .billing_center
        .unwrap_or("EU".to_string())
        .clone();
    sqlx::query!(
        r#"
         INSERT INTO
            instances(
            client_id, plan, 
            display_name, email_domain, 
            region, instance_state, 
            billing_center)
         VALUES
            ( $1, $2, $3, $4, $5, $6, $7)"#,
        client_id,
        plan,
        display_name,
        email_domain,
        region,
        InstanceState::Unassigned.to_string(),
        billing_center
    )
    .execute(conn)
    .await
    .map_err(AuthAppError::SqlError)
}

#[post("")]
async fn create_instance(
    body: web::Json<NewInstanceBody>,
    conn: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, AuthAppError> {
    create_instance_db(body.into_inner(), conn.as_ref()).await?;
    Ok(HttpResponse::Created().finish())
}
#[get("")]
async fn list_instances(conn: web::Data<Pool<Postgres>>) -> Result<HttpResponse, AuthAppError> {
    let instances: Vec<InstanceInfo> = sqlx::query_as!(InstanceInfo, "SELECT * FROM instances")
        .fetch_all(conn.as_ref())
        .await
        .map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Ok().json(instances))
}

#[post("/{client_id}/assign")]
async fn assign_instance(
    client_id: web::Path<PathInfo>,
    assign_body: web::Json<AssignInstance>,
    conn: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, AuthAppError> {
    let state = assign_body
        .instance_state
        .clone()
        .unwrap_or(InstanceState::Trial)
        .to_string();
    let trial_start = Utc::now();
    let trial_expiry = trial_start + Duration::days(14);
    let display_name = assign_body.display_name.clone();
    let email_domain = assign_body.email_domain.clone();
    let client = client_id.into_inner().client_id.clone();
    sqlx::query!(
        r#"
        UPDATE instances SET
            instance_state = $1,
            trial_start = $2,
            trial_expiry = $3,
            display_name = $4,
            email_domain = $5
        WHERE
            client_id = $6

    "#,
        state,
        trial_start,
        trial_expiry,
        display_name,
        email_domain,
        client
    )
    .execute(conn.as_ref())
    .await
    .map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn configure_instance_services(cfg: &mut web::ServiceConfig) {
    cfg.service(list_instances)
        .service(create_instance)
        .service(assign_instance);
}
