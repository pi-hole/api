// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Web Interface Config
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use std::{ffi::OsStr, path::PathBuf};

/// Configuration settings for hosting the web interface
#[derive(Deserialize, Clone, Debug)]
pub struct WebConfig {
    /// If the web interface should be hosted
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// If the redirect from `/` to the web interface path should be enabled
    #[serde(default = "default_root_redirect")]
    pub root_redirect: bool,

    /// The path to mount the web interface on
    #[serde(default = "default_path")]
    pub path: PathBuf
}

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig {
            enabled: default_enabled(),
            root_redirect: default_root_redirect(),
            path: default_path()
        }
    }
}

impl WebConfig {
    pub fn is_valid(&self) -> bool {
        self.path.is_absolute()
    }

    /// Get the web mount path with a trailing slash
    pub fn path_with_trailing_slash(&self) -> String {
        if self.path == OsStr::new("/") {
            "/".to_owned()
        } else {
            self.path.to_string_lossy().into_owned() + "/"
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_root_redirect() -> bool {
    true
}

fn default_path() -> PathBuf {
    PathBuf::from("/admin")
}

#[cfg(test)]
mod test {
    use super::WebConfig;
    use std::path::PathBuf;

    /// The default config is valid
    #[test]
    fn valid_web() {
        let web_config = WebConfig::default();

        assert!(web_config.is_valid())
    }

    /// Using a non-absolute path makes the config invalid
    #[test]
    fn invalid_web_path() {
        let web_config = WebConfig {
            path: PathBuf::from("admin/"),
            ..WebConfig::default()
        };

        assert!(!web_config.is_valid());
    }
}
