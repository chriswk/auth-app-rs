use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Apiv2Schema)]
pub struct VersionInfo {
    pub is_debug: bool,
    pub is_release: bool,
    pub package_info: PackageInfo,
    pub git_info: GitInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, Apiv2Schema)]
pub struct PackageInfo {
    pub version: String,
    pub description: String,
    pub major: String,
    pub minor: String,
    pub patch: String,
    pub pre: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Apiv2Schema)]
pub struct GitInfo {
    pub sha: String,
}
