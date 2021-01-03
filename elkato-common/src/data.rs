use chrono::DateTime;
use chrono::Utc;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Booking {
    pub id: String,
    pub resource: String,
    pub user: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Booking {
    pub fn is_active(&self, now: &DateTime<Utc>) -> bool {
        now >= &self.start && now <= &self.end
    }
}
