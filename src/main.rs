use actix_web::{guard, http::header::ContentType, web, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;

use auth_app_rs::{controllers, AppConfig, GitInfo, PackageInfo, VersionInfo};

use shadow_rs::shadow;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;

shadow!(build);

fn get_version_info() -> auth_app_rs::VersionInfo {
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
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
        let shared_s = shared_config.shared_secret;
        let shared_secret_guard =
            guard::fn_guard(move |ctx| match ctx.head().headers().get("Authorization") {
                Some(hv) => hv
                    .to_str()
                    .map(|f| f.to_string().eq(&shared_s))
                    .unwrap_or(false),
                None => false,
            });
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
