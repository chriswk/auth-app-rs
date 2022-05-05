use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserInsert {
    email: String,
    name: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Access {
    client_id: String,
    role: String,
    url: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserAccess {
    client_id: String,
    email: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserAccessWithSigninUrl {
    pub client_id: String,
    pub email: String,
    pub role: String,
    pub sign_in_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct UserInstanceAccess {
    pub client_id: String,
    pub email: String,
    pub role: String,
    pub region: String,
}

pub(crate) trait FindSigninUrl {
    fn find_sign_in_url(self, self_url: String) -> UserAccessWithSigninUrl;
}

impl FindSigninUrl for UserInstanceAccess {
    fn find_sign_in_url(self, self_url: String) -> UserAccessWithSigninUrl {
        UserAccessWithSigninUrl {
            client_id: self.client_id.clone(),
            email: self.email,
            role: self.role,
            sign_in_url: get_instance_url(self_url, self.region.clone(), self.client_id),
        }
    }
}

fn get_instance_url(self_url: String, region: String, client_id: String) -> String {
    match region.as_str() {
        "eu2" => format!("https://eu.{}/{}", self_url, client_id),
        "us" => format!("https://us.{}/{}", self_url, client_id),
        _ => format!("https://{}/{}", self_url, client_id),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FindClientRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserListItem {
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CreateUserRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserPasswordReset {
    email: String,
    name: String,
    password_reset_token: String,
    password_reset_expires: DateTime<Utc>,
    login_attempts: u32,
}

#[cfg(test)]
mod tests {
    use crate::model::{FindSigninUrl, UserInstanceAccess};

    #[test]
    fn can_get_sign_in_url_from_base_url() {
        let blue = UserInstanceAccess {
            client_id: String::from("blue"),
            region: String::from("eu"),
            email: String::from("some@some.com"),
            role: String::from("client"),
        };
        assert_eq!(
            "https://app.unleash-hosted.com/blue",
            blue.find_sign_in_url(String::from("app.unleash-hosted.com"))
                .sign_in_url,
        )
    }
}
