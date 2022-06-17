use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct CreateUserBody {
    pub client_id: String,
    pub email: String,
    #[serde(default = "default_user_role")]
    pub role: Role,
    #[serde(default = "bool::default")]
    pub notify_instance: bool,
}

pub fn default_user_role() -> Role {
    Role::WRITER
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub enum Role {
    WRITER,
    WRITE,
    ADMIN,
}
