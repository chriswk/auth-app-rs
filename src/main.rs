use actix_web::{middleware, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;
use dashmap::DashMap;
use middleware::NormalizePath;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse};
use oauth2::url::Url;
use oauth2::{AuthUrl, Client, ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, RedirectUrl, RevocationUrl, StandardRevocableToken, TokenUrl};
use paperclip::actix::{
    web::{self},
    OpenApiExt,
};
use paperclip::v2::models::{DefaultApiRaw, Info, Tag};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;

use auth_app_rs::version::get_version_info;
use auth_app_rs::{controllers, AppConfig, AppState};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    let init_config = AppConfig::parse();
    let app_config = init_config.clone();
    let port_config = init_config.clone();
    let migration_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(init_config.database_url.as_str())
        .await
        .expect("Couldn't connect to database");
    auth_app_rs::migrate_db(migration_pool).await;
    let labels = HashMap::<String, String>::new();
    let metrics = PrometheusMetricsBuilder::new("authapp")
        .endpoint("/internal-backstage/metrics")
        .const_labels(labels)
        .build()
        .unwrap();
    let mut spec = DefaultApiRaw::default();
    let version_info = get_version_info();
    spec.info = Info {
        version: version_info.package_info.version,
        title: "Auth-app".to_string(),
        description: Some("Instance management for Unleash".to_string()),
        contact: None,
        ..Default::default()
    };
    spec.tags = vec![Tag {
        name: "admin".to_string(),
        description: Some("Admin operations".to_string()),
        external_docs: None,
    }];
    let pool = PgPoolOptions::new()
        .max_connections(init_config.database_max_connections)
        .connect(init_config.database_url.as_str())
        .await
        .expect("Couldn't connect to database");

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string());
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string());
    let google_client_id = ClientId::new(init_config.google_client_id);
    let google_client_secret = ClientSecret::new(init_config.google_client_secret);
    let oauth_client: Client<BasicErrorResponse, BasicTokenResponse, BasicTokenIntrospectionResponse, StandardRevocableToken, BasicRevocationErrorResponse, EndpointMaybeSet, EndpointNotSet, EndpointNotSet, EndpointSet, EndpointMaybeSet> = BasicClient::new(
        google_client_id.clone())
        .set_client_secret(google_client_secret)
        .set_auth_uri_option(auth_url.ok())
        .set_token_uri_option(token_url.ok())
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:1500/api/auth/google/callback".to_string())
                .expect("Your redirect URL is bullshit"),
        ).set_revocation_url(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );
    let code_verifiers: Arc<DashMap<String, String>> = Arc::new(DashMap::default());

    HttpServer::new(move || {
        /*        let shared_config = app_config.clone();
        let shared_s = shared_config.shared_secret;
        let shared_secret_guard =
            guard::fn_guard(move |ctx| match ctx.head().headers().get("Authorization") {
                Some(hv) => {
                    debug!("Header {:#?}", hv);
                    hv.to_str()
                        .map(|f| f.to_string().eq(&shared_s))
                        .unwrap_or(false)
                }
                None => false,
            });*/
        App::new()
            .wrap(NormalizePath::trim())
            .app_data(web::Data::new(AppState {
                oauth: oauth_client.clone(),
                scope_url: Url::options()
                    .parse("https://www.googleapis.com/auth/userinfo.email")
                    .expect("Your constants are constantly wrong"),
            }))
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(code_verifiers.clone()))
            .wrap_api_with_spec(spec.clone())
            .with_json_spec_at("/api/spec/v2")
            .with_json_spec_v3_at("/api/spec/v3")
            .with_swagger_ui_at("/swagger-ui")
            .wrap(metrics.clone())
            .service(
                web::scope("/internal-backstage")
                    .configure(controllers::internalbackstage::configure_internal_backstage),
            )
            .service(web::scope("/api").configure(controllers::api::configure_api))
            .build()
    })
        .bind(("0.0.0.0", port_config.port))?
        .run()
        .await
}
