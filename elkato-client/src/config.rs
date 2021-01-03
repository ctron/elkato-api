use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub club: String,
    pub username: String,
    pub password: Option<String>,
}
