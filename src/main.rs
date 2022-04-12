use actix_web::{
    http::header::ContentType, web, App, HttpResponse, HttpResponseBuilder, HttpServer,
};
use config::Config;
use serde::{Deserialize, Serialize};
use shadow_rs::shadow;

shadow!(build);

#[derive(Debug, Default, Deserialize, PartialEq)]
struct AppConfig {
    port: u16,
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
    let config = Config::builder()
        .add_source(
            config::Environment::default()
                .try_parsing(true)
                .list_separator(" "),
        )
        .add_source(config::File::with_name("AuthApp"))
        .build();

    HttpServer::new(|| {
        App::new()
            .route(
                "/healthz",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route(
                "/version",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&get_version_info()).unwrap())
                }),
            )
    })
    .bind(("0.0.0.0", 1500))?
    .run()
    .await
}
