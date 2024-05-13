use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

// enum ConfigError {
//     ConfigError(String),
// }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub rss_config: RSSConfig,
    pub log_config: LogConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogConfig {
    pub log_level: String,
    pub log_file: String,
    pub log_console: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RSSConfig {
    pub list: Vec<RSSItem>,
    pub interval_seconds: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RSSItem {
    pub name: String,
    pub url: String,
    pub active: bool,
}

impl AppConfig {
    fn default() -> Self {
        let rss_config = RSSConfig {
            list: vec![],
            interval_seconds: 900,
        };
        let log_config = LogConfig {
            log_level: "warn".to_string(),
            log_file: "data/log/bangumi007.log".to_string(),
            log_console: true,
        };
        AppConfig {
            rss_config,
            log_config,
        }
    }

    pub fn load() -> Self {
        // Load config from data/config/app_config.toml
        // If the file does not exist, make a new one with default values
        // If parse error, backup the old file and make a new one with default values
        match std::fs::read_to_string("data/config/app_config.toml") {
            Err(_) => {
                let default_config = AppConfig::default();
                std::fs::write("data/config/app_config.toml", toml::to_string(&default_config).unwrap()).unwrap();
                default_config
            }
            Ok(content) => {
                toml::from_str(&content).unwrap_or_else(|_| {
                    let backup_file = format!("data/config/app_config.toml.{}.broken", chrono::Local::now().format("%Y%m%d%H%M%S"));
                    std::fs::rename("data/config/app_config.toml", &backup_file).unwrap();
                    let default_config = AppConfig::default();
                    std::fs::write("data/config/app_config.toml", toml::to_string(&default_config).unwrap()).unwrap();
                    default_config
                })
            }
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) {
        std::fs::write("data/config/app_config.toml", toml::to_string(&self).unwrap()).unwrap();
    }

    #[allow(dead_code)]
    pub fn reset(&self) {
        let default_config = AppConfig::default();
        std::fs::write("data/config/app_config.toml", toml::to_string(&default_config).unwrap()).unwrap();
    }
}

lazy_static! {
    pub static ref CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::load());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        println!("{:?}", CONFIG.read().unwrap().log_config.log_file);
        CONFIG.write().unwrap().rss_config.list.push(RSSItem {
            name: "test1".to_string(),
            url: "https://example1.com".to_string(),
            active: true,
        });
        CONFIG.write().unwrap().rss_config.list.push(RSSItem {
            name: "test2".to_string(),
            url: "https://example2.com".to_string(),
            active: true,
        });
        CONFIG.write().unwrap().save();
        CONFIG.write().unwrap().reset();
    }
    
}