use ahash::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml_edit::DocumentMut;

pub struct Config {
    pub doc:    DocumentMut,
    pub config: HashMap<String, BaseConfig>,
}

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to serialize config file: {0}")]
    SerializationError(#[from] toml::ser::Error),
    #[error("Failed to parse config file: {0}")]
    TomlError(#[from] toml_edit::TomlError),
    #[error("Failed to serialize config file: {0}")]
    TomlSerializationError(#[from] toml_edit::ser::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseConfig {
    pub path:   String,
    #[serde(default = "true_")]
    pub enable: bool,
    #[serde(default)]
    pub args:   Vec<String>,
    pub config: Option<toml::Value>,
}

const fn true_() -> bool {
    true
}

impl Config {
    /// # Errors
    ///
    /// * `ReadError` - Failed to read config file
    /// * `ParseError` - Failed to parse config file
    pub fn load_config(path: &str) -> Result<Self, LoadConfigError> {
        let config_file = std::fs::read_to_string(path)?;
        let config: HashMap<String, BaseConfig> = toml::from_str(&config_file)?;
        let doc = config_file.parse()?;

        Ok(Self { doc, config })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &BaseConfig)> {
        self.config.iter().map(|(key, value)| (key.as_str(), value))
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.config.keys()
    }

    pub fn keys_enabled(&self) -> impl Iterator<Item = &String> {
        self.config.iter().filter(|(_, config)| config.enable).map(|(key, _)| key)
    }

    /// # Errors
    ///
    /// * `ReadError` - Failed to read config file
    /// * `ParseError` - Failed to parse config file
    pub fn set(&mut self, key: &str, value: BaseConfig) -> Result<(), LoadConfigError> {
        let doc = toml_edit::ser::to_document(&value)?;
        self.config.insert(key.to_owned(), value);
        self.doc[key] = doc.into_item();
        Ok(())
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&BaseConfig> {
        self.config.get(key)
    }
}
