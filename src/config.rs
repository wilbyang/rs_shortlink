use serde::{Deserialize, Serialize};
use std::fs;
use crate::err::LinkErrors;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub general: GeneralConfig,
    pub db: DbConfig,

}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    pub addr: String,

}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DbConfig {
    pub db_conn: String,

}

impl ServerConfig {
    pub fn load(path: &str) -> Result<Self, LinkErrors> {
        let config = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&config)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_should_be_loaded() {
        let result: Result<ServerConfig, toml::de::Error> =
            toml::from_str(include_str!("../fixtures/server.conf"));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().general.addr, "127.0.0.1:9527");
    }
}