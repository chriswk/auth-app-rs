mod auth;
mod controllers;
mod errors;
mod model;
mod password;
mod user;

use actix_web::{guard, http::header::ContentType, web, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;
use serde::{Deserialize, Serialize};
use shadow_rs::shadow;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;

shadow!(build);

#[derive(Debug, Default, Deserialize, PartialEq, Parser, Clone)]
#[clap(author, version, about, long_about = None)]
struct AppConfig {
    #[clap(short, long, env, default_value_t = 1500, parse(try_from_str))]
    pub port: u16,
    #[clap(short, long, env)]
    pub database_url: String,
    #[clap(short = 'm', long, default_value_t = 2, parse(try_from_str))]
    pub database_max_connections: u32,
    #[clap(short, long, default_value_t = String::from("development"))]
    pub run_mode: String,

    #[clap(short, long, env)]
    pub secret: String,

    #[clap(short = 'h', long)]
    pub shared_secret: String,

    #[clap(short, long, env, default_value_t = String::from("app.unleash-hosted.com"))]
    pub base_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VersionInfo {
    is_debug: bool,
    is_release: bool,
    package_info: PackageInfo,
    git_info: GitInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PackageInfo {
    version: String,
    major: String,
    minor: String,
    patch: String,
    pre: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GitInfo {
    branch: String,
    is_clean: bool,
    sha: String,
    tag: String,
}

fn get_version_info() -> VersionInfo {
    VersionInfo {
        is_debug: shadow_rs::is_debug(),
        is_release: shadow_rs::is_release(),
        package_info: PackageInfo {
            version: build::PKG_VERSION.to_string(),
            major: build::PKG_VERSION_MAJOR.to_string(),
            minor: build::PKG_VERSION_MINOR.to_string(),
            patch: build::PKG_VERSION_PATCH.to_string(),
            pre: build::PKG_VERSION_PRE.to_string(),
        },
        git_info: GitInfo {
            branch: shadow_rs::branch(),
            is_clean: shadow_rs::git_clean(),
            sha: build::COMMIT_HASH.to_string(),
            tag: shadow_rs::tag(),
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
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
        .endpoint("/metrics")
        .const_labels(labels)
        .build()
        .unwrap();

    HttpServer::new(move || {
        let shared_config = app_config.clone();
        let shared_secret_guard = guard::Header("Authorization", shared_config.shared_secret);
        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(metrics.clone())
            .route(
                "/healthz",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(r#"{ "status": "OK" }"#)
                }),
            )
            .route(
                "/internal-backstage/version",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&get_version_info()).unwrap())
                }),
            )
            .service(web::scope("/api/admin/users").configure(user::configure_user_svc))
            .service(
                web::scope("/api/instances")
                    .guard(shared_secret_guard)
                    .configure(controllers::instance_users::configure_instance_user_service),
            )
            .service(
                web::scope("/admin/instances")
                    .configure(controllers::instance::configure_instance_services),
            )
    })
    .bind(("0.0.0.0", port_config.port))?
    .run()
    .await
}
