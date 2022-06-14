use crate::model::version_info::VersionInfo;
use crate::version::get_version_info;
use paperclip::actix::web::Json;
use paperclip::actix::{api_v2_operation, get};

#[api_v2_operation]
#[get("/version")]
pub async fn version() -> Json<VersionInfo> {
    Json(get_version_info())
}
