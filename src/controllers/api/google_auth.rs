use crate::{service, AppConfig, AppState};
use actix_web::cookie::time::Duration;
use actix_web::http::header;
use actix_web::HttpResponse;
use awc::cookie::{Cookie, SameSite};
use log::warn;
use oauth2::http::{HeaderMap, HeaderValue, Method};
use oauth2::reqwest::http_client;
use oauth2::url::Url;
use oauth2::{
    AccessToken, AuthorizationCode, CsrfToken, HttpRequest, PkceCodeChallenge, PkceCodeVerifier,
    Scope, TokenResponse,
};
use paperclip::actix::web::ServiceConfig;
use paperclip::actix::{api_v2_operation, web, Apiv2Schema};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::auth::token;
use crate::errors::AuthAppError;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[api_v2_operation]
async fn login(
    app_state: web::Data<AppState>,
    code_verifiers: web::Data<Mutex<HashMap<String, String>>>,
) -> HttpResponse {
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = &app_state
        .oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let mut code_v = code_verifiers.lock().unwrap();
    let csrf = csrf_state.secret().to_string();
    let verifier = pkce_code_verifier.secret().to_string();
    warn!("csrftoken: {:#?}", csrf);
    warn!("Verifier: {:#?}", verifier);

    code_v.insert(csrf, verifier);
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
    email_verified: bool,
}

fn build_user_info_request(scope_url: Url, access_token: &AccessToken) -> HttpRequest {
    let mut headers = HeaderMap::new();
    let secret =
        HeaderValue::from_str(format!("Bearer {}", access_token.secret()).as_str()).unwrap();
    headers.insert("Authorization", secret);
    HttpRequest {
        url: scope_url,
        headers,
        method: Method::GET,
        body: vec![],
    }
}

fn get_auth_info(url: Url, access_token: &AccessToken) -> Result<AuthInfo, AuthAppError> {
    let req = build_user_info_request(url, access_token);
    http_client(req)
        .map(|r| serde_json::from_slice(r.body.as_slice()).unwrap())
        .map_err(|_| AuthAppError::AccessNotAllowed)
}

#[api_v2_operation]
async fn callback(
    conn: web::Data<Pool<Postgres>>,
    data: web::Data<AppState>,
    config: web::Data<AppConfig>,
    params: web::Query<AuthRequest>,
    code_verifiers: web::Data<Mutex<HashMap<String, String>>>,
) -> Result<HttpResponse, AuthAppError> {
    let code = AuthorizationCode::new(params.code.clone());
    let state = CsrfToken::new(params.state.clone());
    let _scope = params.scope.clone();
    let cv = code_verifiers.lock().unwrap();
    warn!("Getting {:#?}", state.secret());
    let verifier = cv.get(state.secret());
    match verifier {
        Some(v) => {
            let verifier = PkceCodeVerifier::new(v.to_string());

            let token = &data
                .oauth
                .exchange_code(code)
                .set_pkce_verifier(verifier)
                .request(http_client);
            match token {
                Ok(btr) => {
                    warn!("Made token request {:#?}", btr);
                    let auth_info = get_auth_info(data.scope_url.clone(), btr.access_token());
                    match auth_info {
                        Ok(auth) => {
                            warn!("Got authinfo: {:#?}", auth);
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
                        Err(_) => Err(AuthAppError::AccessNotAllowed),
                    }
                }
                Err(_) => Err(AuthAppError::AccessNotAllowed),
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
