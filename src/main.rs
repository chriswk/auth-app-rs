use actix_web::{
    App, http::header::ContentType, HttpResponse, HttpServer, web,
};
use serde::{Deserialize, Serialize};
use shadow_rs::shadow;
use sqlx::postgres::PgPoolOptions;
use clap::Parser;

shadow!(build);

#[derive(Debug, Default, Deserialize, PartialEq, Parser)]
#[clap(author, version, about, long_about = None)]
struct AppConfig {
    #[clap(short, long, env, default_value_t = 1500, parse(try_from_str))]
    port: u16,
    #[clap(short, long, env)]
    database_url: String,
    #[clap(short = 'm', long, default_value_t = 2, parse(try_from_str))]
    database_max_connections: u32,
    #[clap(short, long, default_value_t = String::from("development"))]
    run_mode: String,

    #[clap(short, long, env)]
    secret: String
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
    let pool = PgPoolOptions::new()
        .max_connections(app_config.database_max_connections)
        .connect(app_config.database_url.as_str())
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!().run(&pool).await.expect("Failed to migrate");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
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
    })
    .bind(("0.0.0.0", app_config.port))?
    .run()
    .await
}
