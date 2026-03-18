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

#[cfg(test)]
mod tests {
    use super::*;

    // --- PatternConfig::apply ---

    #[test]
    fn match_variant_applies_regex() {
        let p = PatternConfig::Match {
            pattern: r"foo".to_string(),
            replacement: "bar".to_string(),
        };
        assert_eq!(p.apply("foo baz").unwrap(), Some("bar baz".to_string()));
    }

    #[test]
    fn match_variant_returns_none_on_no_match() {
        let p = PatternConfig::Match {
            pattern: r"foo".to_string(),
            replacement: "bar".to_string(),
        };
        assert_eq!(p.apply("qux").unwrap(), None);
    }

    #[test]
    fn domain_variant_applies_domain_swap() {
        let p = PatternConfig::Domain {
            source: "x.com".to_string(),
            target: "example.com".to_string(),
        };
        assert_eq!(
            p.apply("https://x.com/path").unwrap(),
            Some("https://example.com/path".to_string())
        );
    }

    #[test]
    fn domain_variant_returns_none_on_no_match() {
        let p = PatternConfig::Domain {
            source: "x.com".to_string(),
            target: "example.com".to_string(),
        };
        assert_eq!(p.apply("https://twitter.com/path").unwrap(), None);
    }

    #[test]
    fn nop_variant_returns_input_unchanged() {
        assert_eq!(
            PatternConfig::Nop.apply("hello world").unwrap(),
            Some("hello world".to_string())
        );
    }

    // --- Config::apply ---

    #[test]
    fn config_apply_returns_none_when_no_patterns_match() {
        let config = Config {
            patterns: vec![PatternConfig::Domain {
                source: "x.com".to_string(),
                target: "example.com".to_string(),
            }],
            tick_interval_ms: 100,
        };
        assert_eq!(config.apply("https://twitter.com/foo"), None);
    }

    #[test]
    fn config_apply_returns_first_match() {
        let config = Config {
            patterns: vec![
                PatternConfig::Domain {
                    source: "x.com".to_string(),
                    target: "first.com".to_string(),
                },
                PatternConfig::Domain {
                    source: "x.com".to_string(),
                    target: "second.com".to_string(),
                },
            ],
            tick_interval_ms: 100,
        };
        assert_eq!(
            config.apply("https://x.com/path"),
            Some("https://first.com/path".to_string())
        );
    }

    #[test]
    fn config_apply_skips_patterns_that_produce_identical_output() {
        // Nop returns Some(haystack) but the guard `replacement != haystack` skips it.
        // The Domain pattern after it should still fire.
        let config = Config {
            patterns: vec![
                PatternConfig::Nop,
                PatternConfig::Domain {
                    source: "x.com".to_string(),
                    target: "example.com".to_string(),
                },
            ],
            tick_interval_ms: 100,
        };
        assert_eq!(
            config.apply("https://x.com/path"),
            Some("https://example.com/path".to_string())
        );
    }

    #[test]
    fn config_apply_returns_none_for_empty_patterns() {
        let config = Config {
            patterns: vec![],
            tick_interval_ms: 100,
        };
        assert_eq!(config.apply("anything"), None);
    }

    // --- Config::default ---

    #[test]
    fn default_config_has_expected_tick_interval() {
        assert_eq!(Config::default().tick_interval_ms, 100);
    }

    #[test]
    fn default_config_redirects_x_com() {
        let result = Config::default().apply("https://x.com/user/status/123");
        assert_eq!(result, Some("https://stupidpenisx.com/user/status/123".to_string()));
    }

    #[test]
    fn default_config_normalises_youtube_url() {
        let result = Config::default()
            .apply("https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=related");
        assert_eq!(result, Some("https://youtube.com/watch?v=dQw4w9WgXcQ".to_string()));
    }
}
