use serde::{Deserialize, Serialize};

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
