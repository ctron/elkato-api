use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub url: Url,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct User {
    pub club: String,
    pub username: String,
    pub password: Option<String>,
}
