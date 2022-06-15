use std::collections::HashMap;

use actix_web::{middleware, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;
use middleware::NormalizePath;

use paperclip::actix::{
    web::{self},
    OpenApiExt,
};
use paperclip::v2::models::{DefaultApiRaw, Info, Tag};
use sqlx::postgres::PgPoolOptions;

use auth_app_rs::version::get_version_info;
use auth_app_rs::{controllers, AppConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let app_config = AppConfig::parse();
    let port_config = app_config.clone();
    let pool = PgPoolOptions::new()
        .max_connections(app_config.database_max_connections)
        .connect(app_config.database_url.as_str())
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate");
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
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(pool.clone()))
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
