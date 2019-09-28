// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// LDAP Config
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

/// Configuration settings for LDAP authentication
#[derive(Deserialize, Clone, Debug)]
pub struct LdapConfig {
    /// If LDAP should be enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// LDAP server address
    #[serde(default = "default_address")]
    pub address: String,

    /// Bind Dn
    #[serde(default = "default_bind_dn")]
    pub bind_dn: String
}

impl Default for LdapConfig {
    fn default() -> Self {
        LdapConfig {
            enabled: default_enabled(),
            address: default_address(),
            bind_dn: default_bind_dn()
        }
    }
}

impl LdapConfig {
    pub fn is_valid(&self) -> bool {
        true
    }
}

fn default_enabled() -> bool {
    false
}

fn default_address() -> String {
    "ldap://localhost:389".to_owned()
}

fn default_bind_dn() -> String {
    "".to_owned()
}

#[cfg(test)]
mod test {
    use super::LdapConfig;

    /// The default config is valid
    #[test]
    fn valid_ldap() {
        let ldap_config = LdapConfig::default();

        assert!(ldap_config.is_valid())
    }

}
