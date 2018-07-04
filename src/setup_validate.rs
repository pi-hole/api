/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Validation tests for setupVars.conf & pihole-FTL.conf entries
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use regex::Regex;

/// Contains per entry checks for allowable values of setupVars.conf entries
#[allow(unused)]
pub fn validate_setup_vars(entry: &str, setting: &str) -> bool {
    match entry {
        // Alphanumeric word
        "PIHOLE_INTERFACE" => {
            let domain = Regex::new(r"^([a-zA-Z]|[a-zA-Z0-9][a-zA-Z0-9]*[a-zA-Z0-9])$").unwrap();
            domain.is_match(&setting)
        },
        // Alphanumeric (or hyphenated) word (or null string)
        "CONDITIONAL_FORWARDING_DOMAIN" | "PIHOLE_DOMAIN" => {
            if setting == "" { return true };
            let domain = Regex::new(r"^([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])$").unwrap();
            domain.is_match(&setting)
        },
        // Booleans (or null string)
        "API_PRIVACY_MODE" | "CONDITIONAL_FORWARDING" |
        "DHCP_ACTIVE" | "DHCP_IPv6" | "DNS_BOGUS_PRIV" | "DNS_FQDN_REQUIRED" |
        "DNSSEC" | "INSTALL_WEB_INTERFACE" | "INSTALL_WEB_SERVER" | "QUERY_LOGGING" |
        "WEB_ENABLED" => { // WEB_ENABLED as replacement for LIGHTTPD_ENABLED
            match setting {
                "true"|"false"|"" => true,
                _ => false
            }
        },
        // Integer - One to four digits
        "DHCP_LEASETIME" => {
            let lease_time = Regex::new(r"^\d{1,4}$").unwrap();
            lease_time.is_match(&setting)
        },
        // IPv4 - 4 octets (or null string)
        "DHCP_END" | "DHCP_ROUTER" | "DHCP_START" | 
        "CONDITIONAL_FORWARDING_IP" => {
            if setting == "" { return true };
            let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
            ipv4.is_match(&setting)
        },
        // IPv4 - 4 octets, with mask
        "IPV4_ADDRESS" => { 
            let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)/[0-9]+$").unwrap();
            ipv4.is_match(&setting)
        },
        // IPv6 addresses (or null string)
        "IPV6_ADDRESS" => {
            if setting == "" { return true };
            let ipv6 = Regex::new(r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))").unwrap();
            ipv6.is_match(&setting)
        },
        // Specific test - Query logging options
        "API_QUERY_LOG_SHOW" => {
            match setting {
                "all"|"" => return true,
                _ => return false
            }
        },
        // Specific test - Conditional forwarding reverse domain
        "CONDITIONAL_FORWARDING_REVERSE" => {
            let reverse = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}([a-zA-Z0-9\-\.])+$").unwrap();
                reverse.is_match(&setting)
            },
        // Specific test - Dnsmasq listening options
        "DNSMASQ_LISTENING" => {
            match setting {
                "all"|"lan"|"single"|"" => return true,
                _ => return false
            }
        },
        // Specific test - Boxed Layout
        "WEBUIBOXEDLAYOUT" => {
            match setting {
                "boxed"|"" => return true,
                _ => return false
            }
        },
        // Webpassword prohibited
        "WEBPASSWORD" => false, 
        _ => {
            let pihole_dns = Regex::new(r"^PIHOLE_DNS_[0-9]+$").unwrap();
            // IPv4 address, unmasked.  
            if pihole_dns.is_match(&entry) {
                let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
                return ipv4.is_match(&setting)
            }
            false 
        }
    }
}

/// Contains per entry checks for allowable values of FTL.conf entries
#[allow(unused)]
pub fn validate_ftl_config(entry: &str, setting: &str) -> bool {
    match entry {
        // yes or no will do
        "AAAA_QUERY_ANALYSIS" | "IGNORE_LOCALHOST" | "RESOLVE_IPV4" |
        "RESOLVE_IPV6" | "QUERY_DISPLAY" => { 
            match setting {
                "yes"|"no" => true,
                _ => false
            }
        },
        // Specific to BLOCKINGMODE
        "BLOCKINGMODE" => { 
            match setting {
                "NULL"|"IP-AAAA-NODATA"|"IP"|"NXDOMAIN" => true,
                _ => false
            }
        },
        // Filename (or null)
        "DBFILE" => {
            if setting == "" { return true };
            // Filename regex here
            let filename = Regex::new(r"^(/(\S)+)+$").unwrap();
            if filename.is_match(&setting) { return true };
            false
        },
        // Decimal
        "DBINTERVAL" | "MAXLOGAGE" => {
            let decimal = Regex::new(r"^(\d)+(\.)?(\d)*$").unwrap();
            if decimal.is_match(&setting) { return true };
            false 
        },
        // Integer
        "MAXDBDAYS" => {
            let intnum = Regex::new(r"^(\d)+$").unwrap();
            if intnum.is_match(&setting) { return true };
            false
        },
        // Port number (0-65535)
        "FTLPORT" => {
            let port = Regex::new(r"^((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$").unwrap();
            if port.is_match(&setting) { return true };
            false
        },
        // Specific to PRIVACYLEVEL
        "PRIVACYLEVEL" => { 
            match setting {
                "0"|"1"|"2"|"3" => true,
                _ => false
            }
        },
        // Specific to SOCKET_LISTENING
        "SOCKET_LISTENING" => {
            match setting {
                "localonly"|"all" => true,
                _ => false
            }
        },
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use setup_validate::{validate_setup_vars, validate_ftl_config};

    #[test]
    fn test_validate_setup_vars_valid() {
        let tests = [
            // Acceptable parameters
            ("API_QUERY_LOG_SHOW", "all", true),
            ("API_PRIVACY_MODE", "false", true),
            ("DNS_BOGUS_PRIV", "true", true),
            ("DNS_FQDN_REQUIRED", "true", true),
            ("CONDITIONAL_FORWARDING", "true", true),
            ("CONDITIONAL_FORWARDING_DOMAIN", "hub", true),
            ("CONDITIONAL_FORWARDING_IP", "192.168.1.1", true),
            ("CONDITIONAL_FORWARDING_REVERSE", "1.168.192.in-addr.arpa", true),
            ("DHCP_ACTIVE", "false", true),
            ("DHCP_END", "199.199.1.255", true),
            ("DHCP_IPv6", "false", true),
            ("DHCP_LEASETIME", "24", true),
            ("DHCP_START", "199.199.1.0", true),
            ("DHCP_ROUTER", "192.168.1.1", true),
            ("DNSMASQ_LISTENING", "all", true),
            ("DNSSEC", "false", true),
            ("INSTALL_WEB_SERVER", "true", true),
            ("INSTALL_WEB_INTERFACE", "true", true),
            ("IPV4_ADDRESS", "192.168.1.205/24", true),
            ("IPV6_ADDRESS", "2001:470:66:d5f:114b:a1b9:2a13:c7d9", true),
            ("PIHOLE_DNS_1", "8.8.8.8", true),
            ("PIHOLE_DNS_2", "8.8.4.4", true),
            ("PIHOLE_DOMAIN", "lan", true),
            ("PIHOLE_INTERFACE", "enp0s3", true),
            ("QUERY_LOGGING", "true", true),
            ("WEBUIBOXEDLAYOUT", "boxed", true),
            ("WEB_ENABLED", "false", true)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_setup_vars(setting, value), result);
        }
    }

    #[test]
    fn test_validate_setup_vars_invalid() {
        let tests = [
            // Nonsensical parameters
            ("API_QUERY_LOG_SHOW", "41", false),
            ("API_PRIVACY_MODE", "off", false),
            ("DNS_BOGUS_PRIV", "on", false),
            ("DNS_FQDN_REQUIRED", "probably", false),
            ("CONDITIONAL_FORWARDING", "disabled", false),
            ("CONDITIONAL_FORWARDING_DOMAIN", "%%@)#", false),
            ("CONDITIONAL_FORWARDING_IP", "192.1.1", false),
            ("CONDITIONAL_FORWARDING_REVERSE", "in-addr.arpa.1.1.1", false),
            ("DHCP_ACTIVE", "active", false),
            ("DHCP_END", "2001:470:66:d5f:114b:a1b9:2a13:c7d9", false),
            ("DHCP_IPv6", "ipv4", false),
            ("DHCP_LEASETIME", "hours", false),
            ("DHCP_START", "199199.1.0", false),
            ("DHCP_ROUTER", "192.1681.1", false),
            ("DNSMASQ_LISTENING", "dnsmasq", false),
            ("DNSSEC", "enabled", false),
            ("INSTALL_WEB_SERVER", "yes", false),
            ("INSTALL_WEB_INTERFACE", "no", false),
            ("IPV4_ADDRESS", "192.168.1.205", false),
            ("IPV6_ADDRESS", "192.168.1.205", false),
            ("PIHOLE_DNS_1", "www.pi-hole.net", false),
            ("PIHOLE_DNS_2", "4.5", false),
            ("PIHOLE_DOMAIN", "too many words", false),
            ("PIHOLE_INTERFACE", "/dev/net/eth1", false),
            ("QUERY_LOGGING", "disabled", false),
            ("WEBUIBOXEDLAYOUT", "unboxed", false),
            ("WEB_ENABLED", "457", false)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_setup_vars(setting, value), result);
        }
    }

    #[test]
    fn test_validate_setup_vars_disabled() {
        // Disabled / disallowed options 
        // Webpassword disallowed - must report false.
        assert_eq!(validate_setup_vars("WEBPASSWORD", "B350486529B6022919491965A235157110B12437514315201184143ABBB37A14"), false);
    }

    #[test]
    fn test_validate_ftl_conf_valid() {
        let tests = [
            // Acceptable paramaters
            ("SOCKET_LISTENING", "localonly", true),
            ("QUERY_DISPLAY", "yes", true),
            ("AAAA_QUERY_ANALYSIS", "no", true),
            ("RESOLVE_IPV6", "yes", true),
            ("RESOLVE_IPV4", "no", true),
            ("MAXDBDAYS", "3", true),
            ("DBINTERVAL", "5.0", true),
            ("DBFILE", "/etc/pihole/FTL.conf", true),
            ("MAXLOGAGE", "8", true),
            ("FTLPORT", "64738", true),
            ("PRIVACYLEVEL", "2", true),
            ("IGNORE_LOCALHOST", "yes", true),
            ("BLOCKINGMODE", "NULL", true)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_ftl_config(setting, value), result);
        }
    }

    #[test]
    fn test_validate_ftl_conf_invalid() {
        let tests = [
            // Nonsensical parameters
            ("SOCKET_LISTENING", "5", false),
            ("QUERY_DISPLAY", "true", false),
            ("AAAA_QUERY_ANALYSIS", "", false),
            ("RESOLVE_IPV6", "-1", false),
            ("RESOLVE_IPV4", "127.0.0.1", false),
            ("MAXDBDAYS", "nine", false),
            ("DBINTERVAL", "5.0.0", false),
            ("DBFILE", "http://www.pi-hole.net", false),
            ("MAXLOGAGE", "enabled", false),
            ("FTLPORT", "any", false),
            ("PRIVACYLEVEL", "high", false),
            ("IGNORE_LOCALHOST", "127.0.0.1", false),
            ("BLOCKINGMODE", "enabled", false)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_ftl_config(setting, value), result);
        }
    }
}
