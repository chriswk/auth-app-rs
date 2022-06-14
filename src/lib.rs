pub mod controllers;
pub(crate) mod db;
mod errors;
mod model;
pub mod version;
use clap::Parser;
use serde::Deserialize;

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

    #[clap(short, long, env, default_value_t = String::from("app.unleash-hosted.com"))]
    pub base_url: String,
}
