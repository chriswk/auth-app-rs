use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use sqlx::{Encode, FromRow};
use strum::{Display, EnumString};

#[derive(Serialize, Deserialize, Clone, Apiv2Schema)]
pub struct CreateUserBody {
    pub client_id: String,
    pub email: String,
    #[serde(default = "default_user_role")]
    pub role: String,
    #[serde(default = "bool::default")]
    pub notify_instance: bool,
}

#[derive(Serialize, Deserialize, FromRow, Apiv2Schema)]
pub struct MinimalAuthUser {
    pub email: String,
    pub name: Option<String>,
}

pub fn default_user_role() -> String {
    Role::WRITER.to_string()
}

#[derive(Clone, Encode, Display, Debug, Serialize, Deserialize, Apiv2Schema, EnumString)]
pub enum Role {
    WRITER,
    WRITE,
    ADMIN,
}
