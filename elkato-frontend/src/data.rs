use anyhow::{anyhow, Result};
use elkato_client::User;
use serde::{Deserialize, Serialize};
use yew::format::Json;
use yew::services::storage::*;

const KEY: &str = "config";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub user: User,
}

impl Config {
    pub fn store(&self) -> Result<()> {
        let mut storage = StorageService::new(Area::Local).unwrap();
        storage.store(KEY, Json(self));
        Ok(())
    }

    pub fn load() -> Result<Config> {
        let storage = StorageService::new(Area::Local).map_err(|err| anyhow!(err))?;

        storage.restore::<Json<Result<Config>>>(KEY).0
    }
}
