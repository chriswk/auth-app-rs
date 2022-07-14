pub mod controllers;
pub(crate) mod db;
pub mod errors;
pub mod model;
pub mod version;
use clap::Parser;
use oauth2::basic::BasicClient;
use oauth2::url::Url;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

#[derive(Debug, Default, Deserialize, PartialEq, Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct AppConfig {
    #[clap(short, long, env, default_value_t = 1500, value_parser)]
    pub port: u16,
    #[clap(short, long, env)]
    pub database_url: String,
    #[clap(short = 'm', long, default_value_t = 2, value_parser)]
    pub database_max_connections: u32,
    #[clap(short, long, default_value_t = String::from("development"))]
    pub run_mode: String,

    #[clap(short, long, env)]
    pub secret: String,

    #[clap(short = 'h', long)]
    pub shared_secret: String,

    #[clap(long, env, default_value_t = String::from("eu-central-1"))]
    pub aws_region: String,

    #[clap(long, env)]
    pub aws_access_key_id: Option<String>,

    #[clap(long, env)]
    pub aws_secret_access_key: Option<String>,

    #[clap(short, long, env, default_value_t = String::from("app.unleash-hosted.com"))]
    pub base_url: String,

    #[clap(long, env)]
    pub stripe_key: Option<String>,

    #[clap(long, env)]
    pub sendinblue_key: Option<String>,

    #[clap(long, env, default_value_t = false)]
    pub secure: bool,

    #[clap(long, env)]
    pub google_client_id: String,

    #[clap(long, env)]
    pub google_client_secret: String,
}

pub struct AppState {
    pub oauth: BasicClient,
    pub scope_url: Url,
}

pub async fn migrate_db(pool: Pool<Postgres>) {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate");
}
