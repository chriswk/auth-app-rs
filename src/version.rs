use crate::model::version_info::{GitInfo, PackageInfo, VersionInfo};
use shadow_rs::shadow;

shadow!(build);

pub fn get_version_info() -> VersionInfo {
    VersionInfo {
        is_debug: shadow_rs::is_debug(),
        is_release: shadow_rs::is_release(),
        package_info: PackageInfo {
            version: build::PKG_VERSION.to_string(),
            description: build::PKG_DESCRIPTION.to_string(),
            major: build::PKG_VERSION_MAJOR.to_string(),
            minor: build::PKG_VERSION_MINOR.to_string(),
            patch: build::PKG_VERSION_PATCH.to_string(),
            pre: build::PKG_VERSION_PRE.to_string(),
        },
        git_info: GitInfo {
            sha: build::COMMIT_HASH.to_string(),
        },
    }
}
