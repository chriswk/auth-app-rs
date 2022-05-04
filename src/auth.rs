use crate::errors::AuthAppError;
use crate::model::{FindClientRequest, FindSigninUrl, UserAccessWithSigninUrl, UserInstanceAccess};
use crate::AppConfig;
use actix_web::{post, web, HttpResponse};
use sqlx::{Pool, Postgres};

#[post("/sign-in")]
async fn find_client(
    email_request: web::Json<FindClientRequest>,
    conn: web::Data<Pool<Postgres>>,
    conf: web::Data<AppConfig>,
) -> Result<HttpResponse, AuthAppError> {
    let email_lowercase = email_request.email.to_lowercase();
    let instances: Vec<UserInstanceAccess> = sqlx::query_as!(
        UserInstanceAccess,
        r#"
        SELECT u.client_id, u.email, u.role, i.region 
        FROM user_access u 
        JOIN instances i
        ON u.client_id = i.client_id 
        WHERE u.email = $1"#,
        email_lowercase
    )
    .fetch_all(conn.as_ref())
    .await
    .map_err(AuthAppError::SqlError)?;

    let data: Vec<UserAccessWithSigninUrl> = instances
        .iter()
        .map(|i| i.clone().find_sign_in_url(conf.as_ref().base_url.clone()))
        .collect();
    match data.len() {
        0 => Ok(HttpResponse::Unauthorized().finish()),
        1 => {
            let user_access = data.get(0).expect("We just check that length is 1");
            Ok(HttpResponse::Found()
                .insert_header(("redirect", user_access.sign_in_url.as_str()))
                .finish())
        }
        _ => Ok(HttpResponse::Ok().json(&instances)),
    }
}

pub fn configure_auth_services(cfg: &mut web::ServiceConfig) {
    cfg.service(find_client);
}
