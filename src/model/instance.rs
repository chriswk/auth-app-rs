use chrono::{DateTime, Utc};
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(Serialize, Deserialize, Apiv2Schema, FromRow, Debug)]
pub struct InstanceStatus {
    pub plan: String,
    pub trial_expiry: Option<DateTime<Utc>>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_extended: i32,
    pub billing_center: String,
    pub instance_state: String,
    pub region: String,
}

pub struct InstanceRow {
    pub billing_center: String,
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub display_name: Option<String>,
    pub email_domain: Option<String>,
    pub instance_state: String,
    pub plan: String,
    pub region: String,
    pub seats: i32,
    pub stripe_customer_id: Option<String>,
    pub trial_expiry: Option<DateTime<Utc>>,
    pub trial_extended: i32,
    pub trial_start: Option<DateTime<Utc>>,
}

#[derive(
    Display, Debug, Serialize, Deserialize, IntoStaticStr, EnumIter, EnumString, Apiv2Schema,
)]
pub enum InstanceState {
    Unassigned,
    Trial,
    Active,
    Expired,
    Churned,
}
