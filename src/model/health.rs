use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Apiv2Schema)]
pub struct Health {
    pub status: String,
}
