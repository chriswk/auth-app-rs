use crate::auth::token;
use crate::errors::AuthAppError;
use crate::{service, AppConfig, AppState};
use actix_web::cookie::time::Duration;
use actix_web::http::header;
use actix_web::HttpResponse;
use awc::cookie::{Cookie, SameSite};
use dashmap::DashMap;
use log::{info, warn};
use oauth2::reqwest::redirect::Policy;
use oauth2::url::Url;
use oauth2::{
    reqwest, AccessToken, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use paperclip::actix::web::ServiceConfig;
use paperclip::actix::{api_v2_operation, web, Apiv2Schema};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[api_v2_operation]
async fn login(
    app_state: web::Data<AppState>,
    code_verifiers: web::Data<DashMap<String, String>>,
) -> HttpResponse {
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = &app_state
        .oauth
        .authorize_url(CsrfToken::new_random)
        .map(|u| u.add_scope(Scope::new(app_state.scope_url.to_string())))
        .map(|u| u.set_pkce_challenge(pkce_code_challenge))
        .map(|u| u.url()).unwrap();

    let csrf = csrf_state.secret().to_string();
    let verifier = pkce_code_verifier.secret().to_string();
    warn!("csrftoken: {:#?}", csrf);
    warn!("Verifier: {:#?}", verifier);

    code_verifiers.insert(csrf, verifier);
    HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url.to_string()))
        .finish()
}

#[api_v2_operation]
async fn logout() -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, "/".to_string()))
        .finish()
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct AuthRequest {
    code: String,
    state: String,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    email: String,
    verified_email: bool,
}

async fn get_auth_info(http_client: &oauth2::reqwest::Client, url: Url, access_token: &AccessToken) -> Result<AuthInfo, AuthAppError> {
    let result = http_client
        .get(url)
        .bearer_auth(access_token.clone().into_secret())
        .send().await;
    match result {
        Ok(response) => {
            let bytes = response.bytes().await.expect("Failed to get string");
            let slice = bytes.iter().as_slice();
            serde_json::from_slice(&slice).map_err(|_| AuthAppError::AccessNotAllowed)
        }
        Err(_) => {
            info!("Failed to make scope request");
            Err(AuthAppError::AccessNotAllowed)
        }
    }
}

#[api_v2_operation]
async fn callback(
    conn: web::Data<Pool<Postgres>>,
    data: web::Data<AppState>,
    config: web::Data<AppConfig>,
    params: web::Query<AuthRequest>,
    code_verifiers: web::Data<DashMap<String, String>>,
) -> Result<HttpResponse, AuthAppError> {
    let code = AuthorizationCode::new(params.code.clone());
    let state = CsrfToken::new(params.state.clone());
    let _scope = params.scope.clone();
    let http_client: oauth2::reqwest::Client = reqwest::ClientBuilder::new().redirect(Policy::none()).build().expect("Failed to build httpclient");
    let verifier = code_verifiers.get(state.secret());
    match verifier {
        Some(v) => {
            let verifier = PkceCodeVerifier::new(v.to_string());

            let token = &data
                .oauth
                .exchange_code(code).map(|ec| ec.set_pkce_verifier(verifier)).unwrap().request_async(&http_client).await;
            match token {
                Ok(btr) => {
                    warn!("Made token request {:#?}", btr);
                    let auth_info = get_auth_info(&http_client, "https://www.googleapis.com/oauth2/v1/userinfo".parse().expect("Failed to parse info"), btr.access_token()).await;
                    match auth_info {
                        Ok(auth) => {
                            info!("Got authinfo: {:#?}", auth);
                            let auth_app_user =
                                service::user::get_or_create_user(conn.as_ref(), auth.email)
                                    .await?;
                            let token = token::create_token(
                                config.as_ref().clone(),
                                auth_app_user.email,
                                vec![],
                            )?;
                            let cookie_name = config.as_ref().clone().cookie_name;
                            let session_cookie = Cookie::build(cookie_name, token)
                                .max_age(Duration::seconds(config.cookie_life_time_secs))
                                .domain(config.as_ref().clone().cookie_domain)
                                .http_only(true)
                                .same_site(SameSite::Strict)
                                .finish();
                            Ok(HttpResponse::Found()
                                .cookie(session_cookie)
                                .append_header((header::LOCATION, "/admin/instances"))
                                .finish())
                        }
                        Err(_) => {
                            info!("Could not find auth_info in response");
                            Err(AuthAppError::AccessNotAllowed)
                        }
                    }
                }
                Err(_) => {
                    info!("Token request was invalid");
                    Err(AuthAppError::AccessNotAllowed)
                }
            }
        }
        None => {
            println!("Could not find {}", state.secret());
            Err(AuthAppError::AccessNotAllowed)
        }
    }
}

pub fn configure_google_auth(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("/login").route(web::get().to(login)));
    cfg.service(web::resource("/callback").route(web::get().to(callback)));
    cfg.service(web::resource("/logout").route(web::get().to(logout)));
}
