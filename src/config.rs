use libconfig::config;
use serde::{Deserialize, Serialize};

use crate::strategies::{domain::strategy_domain, regex::strategy_regex};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(
    tag = "style",
    rename_all = "snake_case",
    rename_all_fields = "snake_case"
)]
pub enum PatternConfig {
    // https:\/\/x.com\/(.+) => https://stupidpenisx.com/$1
    Match {
        pattern: String,
        replacement: String,
    },
    // x.com -> stupidpenisx.com
    Domain {
        source: String,
        target: String,
    },
    // what
    #[default]
    Nop,
}
impl PatternConfig {
    fn apply(&self, haystack: &str) -> anyhow::Result<Option<String>> {
        match self {
            PatternConfig::Match {
                pattern,
                replacement,
            } => strategy_regex(pattern, replacement, haystack),
            PatternConfig::Domain { source, target } => strategy_domain(source, target, haystack),
            PatternConfig::Nop => Ok(Some(haystack.to_string())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub patterns: Vec<PatternConfig>,
    pub tick_interval_ms: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            patterns: vec![
                PatternConfig::Domain {
                    source: "x.com".to_string(),
                    target: "stupidpenisx.com".to_string(),
                },
                PatternConfig::Match {
                    pattern: "^(.+)v=([a-zA-Z0-9-_]{11})(.+)$".to_string(),
                    replacement: "https://youtube.com/watch?v=$2".to_string(),
                },
            ],
            tick_interval_ms: 100,
        }
    }
}
impl Config {
    pub fn apply(&self, haystack: &str) -> Option<String> {
        for pattern in self.patterns.iter() {
            match pattern.apply(haystack) {
                Ok(Some(replacement)) if replacement != haystack => return Some(replacement),
                Err(e) => todo!("How do I want to handle errors? ({e})"),
                _ => {}
            }
        }

        None
    }
}

config! {
    pub static CLIPD_CONFIG: Config = {
        module: "clipd",
        env_prefix: "CLIPD_",
        impl_trait
    }
}
