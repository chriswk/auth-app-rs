use actix_web::dev::AppConfig;
use actix_web::http::StatusCode;
use actix_web::{get, post, web, HttpResponse, HttpResponseBuilder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

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
}

fn default_plan() -> String {
    "pro".to_string()
}

fn default_region() -> String {
    "eu".to_string()
}

#[derive(Serialize, Deserialize)]
pub struct NewInstanceBody {
    pub client_id: String,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_plan")]
    pub plan: String,
    pub display_name: Option<String>,
    pub email_domain: Option<String>,
}

#[post("")]
async fn create_instance(
    body: web::Json<NewInstanceBody>,
    conn: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, AuthAppError> {
    let client_id = body.client_id.clone();
    let plan = body.plan.clone();
    let display_name = body.display_name.clone();
    let email_domain = body.email_domain.clone();
    let region = body.region.clone();
    sqlx::query!(
        r#"
         INSERT INTO
            instances(client_id, plan, display_name, email_domain, region)
         VALUES
            ( $1, $2, $3, $4, $5 )"#,
        client_id,
        plan,
        display_name,
        email_domain,
        region
    )
    .execute(conn.as_ref())
    .await
    .map_err(AuthAppError::SqlError)?;

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

pub fn configure_instance_services(cfg: &mut web::ServiceConfig) {
    cfg.service(list_instances).service(create_instance);
}
