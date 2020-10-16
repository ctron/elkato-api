
use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct Booking {
    pub id: String,
    pub resource: String,
    pub user: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub description: Option<String>,
}