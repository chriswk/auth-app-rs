pub mod controllers;
mod errors;
mod model;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq, Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct AppConfig {
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
pub struct VersionInfo {
    pub is_debug: bool,
    pub is_release: bool,
    pub package_info: PackageInfo,
    pub git_info: GitInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackageInfo {
    pub version: String,
    pub major: String,
    pub minor: String,
    pub patch: String,
    pub pre: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GitInfo {
    pub branch: String,
    pub is_clean: bool,
    pub sha: String,
    pub tag: String,
}
