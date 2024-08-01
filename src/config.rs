use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum HashAlgorithm {
    MD5,
    SHA256,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::MD5
    }
}


impl ToString for HashAlgorithm {
    fn to_string(&self) -> String {
        match self {
            HashAlgorithm::MD5 => "MD5".to_string(),
            HashAlgorithm::SHA256 => "SHA256".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub hash_algorithm: HashAlgorithm,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hash_algorithm: HashAlgorithm::default(),
        }
    }
}

impl Config {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new(".udv/config");
        let config_content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}
