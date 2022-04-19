use chrono::{DateTime, Utc};

use serde::{Serialize, Deserialize};

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
    name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserListItem {
    pub email: String,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CreateUserRequest {
    pub email: String
}


#[derive(Debug, Serialize, Deserialize)]
struct UserPasswordReset {
    email: String,
    name: String,
    password_reset_token: String,
    password_reset_expires: DateTime<Utc>,
    login_attempts: u32,
}
