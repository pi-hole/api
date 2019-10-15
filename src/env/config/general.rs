// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// General Config
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use rocket::logger::LoggingLevel;
use serde::{Deserialize, Deserializer};
use std::{net::Ipv4Addr, str::FromStr};

/// General config settings
#[derive(Deserialize, Clone, Debug)]
pub struct General {
    /// The address to host the API on
    #[serde(default = "default_address")]
    pub address: String,

    /// The port to host the API on
    #[serde(default = "default_port")]
    pub port: usize,

    /// The log level to use
    #[serde(
        default = "default_log_level",
        deserialize_with = "deserialize_logging_level"
    )]
    pub log_level: LoggingLevel
}

impl Default for General {
    fn default() -> Self {
        General {
            address: default_address(),
            port: default_port(),
            log_level: default_log_level()
        }
    }
}

impl General {
    pub fn is_valid(&self) -> bool {
        Ipv4Addr::from_str(&self.address).is_ok() && self.port <= 65535
    }
}

/// Deserialize a logging level. `LoggingLevel` does not implement
/// `Deserialize`, so this must be plugged in via an attribute on the log level
/// field.
fn deserialize_logging_level<'de, D>(deserializer: D) -> Result<LoggingLevel, D::Error>
where
    D: Deserializer<'de>
{
    let level_str = String::deserialize(deserializer)?;
    LoggingLevel::from_str(&level_str).map_err(serde::de::Error::custom)
}

fn default_address() -> String {
    "0.0.0.0".to_owned()
}

fn default_port() -> usize {
    80
}

fn default_log_level() -> LoggingLevel {
    LoggingLevel::Critical
}

#[cfg(test)]
mod test {
    use super::General;

    /// The default general config is valid
    #[test]
    fn valid_general() {
        let general = General::default();

        assert!(general.is_valid());
    }

    /// An invalid address flags the config as invalid
    #[test]
    fn invalid_general_address() {
        let general = General {
            address: "hello_world".to_owned(),
            ..General::default()
        };

        assert!(!general.is_valid());
    }

    /// An invalid port flags the config as invalid
    #[test]
    fn invalid_general_port() {
        let general = General {
            port: 65536,
            ..General::default()
        };

        assert!(!general.is_valid());
    }
}
