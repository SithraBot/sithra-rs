use std::path::{Path, PathBuf};

use ahash::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml_edit::DocumentMut;

pub struct Config {
    pub ref_path: PathBuf,
    pub doc:      DocumentMut,
    pub config:   HashMap<String, BaseConfig>,
    pub path:     PathBuf,
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
    #[error("Plugin not exists, id: {0}")]
    PluginNotExists(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseConfig {
    pub path:       String,
    #[serde(default = "true_")]
    pub enable:     bool,
    #[serde(default)]
    pub args:       Vec<String>,
    #[serde(rename = "$")]
    pub ref_:       Option<String>,
    pub config:     Option<toml::Value>,
    #[serde(skip)]
    pub raw_config: Option<toml_edit::DocumentMut>,
}

const fn true_() -> bool {
    true
}

impl Config {
    /// # Errors
    ///
    /// * `ReadError` - Failed to read config file
    /// * `ParseError` - Failed to parse config file
    pub fn load_config(
        path: impl AsRef<Path>,
        ref_path: impl AsRef<Path>,
    ) -> Result<Self, LoadConfigError> {
        std::fs::create_dir_all(ref_path.as_ref()).ok();
        let config_file = std::fs::read_to_string(&path)?;
        let mut config: HashMap<String, BaseConfig> = toml::from_str(&config_file)?;
        for (k, v) in &mut config {
            let file_path = if let Some(ref_) = &v.ref_ {
                ref_path.as_ref().join(ref_)
            } else {
                ref_path.as_ref().join(format!("{k}.toml"))
            };
            if file_path.exists() {
                let file = std::fs::read_to_string(&file_path)?;
                let config = toml::from_str(&file)?;
                v.config = config;
                let doc = file.parse()?;
                v.raw_config = Some(doc);
            }
        }
        let doc: DocumentMut = config_file.parse()?;
        log::trace!("{doc}");
        Ok(Self {
            doc,
            config,
            ref_path: ref_path.as_ref().into(),
            path: path.as_ref().into(),
        })
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
    pub fn set_config(&mut self, id: &str, config: &str) -> Result<(), LoadConfigError> {
        let value: toml::Value = toml::from_str(config)?;
        let doc: DocumentMut = config.parse()?;
        if let Some(base_config) = self.config.get_mut(id) {
            base_config.config = Some(value);
            base_config.raw_config = Some(doc);
            Ok(())
        } else {
            Err(LoadConfigError::PluginNotExists(id.to_owned()))
        }
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&BaseConfig> {
        self.config.get(key)
    }

    /// # Errors
    ///
    /// * `std::io::Error` - Failed to write config file
    pub async fn flush_base(&self) -> Result<(), std::io::Error> {
        let content = self.doc.to_string();
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// * `std::io::Error` - Failed to write config file
    pub async fn flush_raw_all(&self) -> Result<(), std::io::Error> {
        for k in self.config.keys() {
            self.flush_raw(k).await?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// * `std::io::Error` - Failed to write config file
    pub async fn flush_raw(&self, id: &str) -> Result<(), std::io::Error> {
        if let Some(config) = self.config.get(id) {
            let file_path = if let Some(ref ref_) = config.ref_ {
                self.ref_path.join(format!("{ref_}.toml"))
            } else {
                self.ref_path.join(format!("{id}.toml"))
            };
            if let Some(ref raw_config) = config.raw_config {
                tokio::fs::write(file_path, raw_config.to_string()).await?;
            }
        }
        Ok(())
    }

    pub fn set_enable(&mut self, id: &str, enable: bool) {
        if let Some(config) = self.config.get_mut(id) {
            config.enable = enable;
            self.doc[id]["enable"] = enable.into();
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<(BaseConfig, toml_edit::Item)> {
        let base = self.config.remove(id);
        let doc = self.doc.remove(id);
        if let (Some(base), Some(doc)) = (base, doc) {
            return Some((base, doc));
        }
        None
    }

    /// # Errors
    ///
    /// * `std::io::Error` - Failed to delete config file
    pub async fn delete_file(&self, id: &str) -> Result<(), std::io::Error> {
        let file_path = if let Some(BaseConfig {
            ref_: Some(ref_), ..
        }) = self.config.get(id)
        {
            self.ref_path.join(format!("{ref_}.toml"))
        } else {
            self.ref_path.join(format!("{id}.toml"))
        };
        if file_path.exists() {
            tokio::fs::remove_file(file_path).await?;
        }
        Ok(())
    }

    pub fn duplicate(&mut self, id: &str, to: &str) {
        let config = self.config.get(id).cloned();
        let item = self.doc.get(id).cloned();
        let Some(mut config) = config else {
            return;
        };
        let Some(mut item) = item else {
            return;
        };
        config.enable = false;
        item["enable"] = false.into();
        self.config.insert(to.to_owned(), config);
        self.doc.insert(to, item);
    }
}
