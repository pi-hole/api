/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Validation tests for setupVars.conf entries
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use regex::Regex;

/// Contains per entry checks for allowable values of setupVars.conf entries
#[allow(unused)]
pub fn validate_setupvars (entry: &str, setting: &str) -> bool {
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
/* Is temperature going to be implemented? - disabled for the moment
        // Unit options, or null
        "TEMPERATUREUNIT" => {
            match setting {
                "C"|"F"|"K"|"" => return true,
                _ => return false
            }
        } */
        _ => {
            let pihole_dns = Regex::new(r"^PIHOLE_DNS_[0-9]+$").unwrap();
            // IPv4 address, unmasked.  
            if pihole_dns.is_match(&entry) {
                let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
                ipv4.is_match(&setting)
                } else {
                false 
            }
        }
    }
}

/// Contains per entry checks for allowable values of FTL.conf entries
#[allow(unused)]
pub fn validate_ftlconf (entry: &str, setting: &str) -> bool {
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
            let filename = Regex::new(r"^(\/(\S)+)+$").unwrap();
            if filename.is_match(&entry) { return true };
            false
        },
        // Decimal
        "DBINTERVAL" | "MAXLOGAGE" => {
            let decimal = Regex::new(r"^(\d)+(\.)?(\d)*$").unwrap();
            if decimal.is_match(&entry) { return true };
            false 
        },
        // Integer
        "MAXDBDAYS" => {
            let intnum = Regex::new(r"^(\d)+$").unwrap();
            if intnum.is_match(&entry) { return true };
            false
        },
        // Port number (0-65535)
        "FTLPORT" => {
            let port = Regex::new(r"^((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$").unwrap();
            if port.is_match(&entry) { return true };
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
    use setup_validate::{validate_setupvars, validate_ftlconf};

    #[test]
    fn test_validatesetupvars() {
        // Acceptable parameters
        assert_eq!(validate_setupvars("API_QUERY_LOG_SHOW","all"), true);
        assert_eq!(validate_setupvars("API_PRIVACY_MODE","false"), true);
        assert_eq!(validate_setupvars("DNS_BOGUS_PRIV","true"), true);
        assert_eq!(validate_setupvars("DNS_FQDN_REQUIRED","true"), true);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING","true"), true);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING_DOMAIN","hub"), true);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING_IP","192.168.1.1"), true);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING_REVERSE","1.168.192.in-addr.arpa"), true);
        assert_eq!(validate_setupvars("DHCP_ACTIVE","false"), true);
        assert_eq!(validate_setupvars("DHCP_END","199.199.1.255"), true);
        assert_eq!(validate_setupvars("DHCP_IPv6","false"), true);
        assert_eq!(validate_setupvars("DHCP_LEASETIME","24"), true);
        assert_eq!(validate_setupvars("DHCP_START","199.199.1.0"), true);
        assert_eq!(validate_setupvars("DHCP_ROUTER","192.168.1.1"), true);
        assert_eq!(validate_setupvars("DNSMASQ_LISTENING","all"), true);
        assert_eq!(validate_setupvars("DNSSEC","false"), true);
        assert_eq!(validate_setupvars("INSTALL_WEB_SERVER","true"), true);
        assert_eq!(validate_setupvars("INSTALL_WEB_INTERFACE","true"), true);
        assert_eq!(validate_setupvars("IPV4_ADDRESS","192.168.1.205/24"), true);
        assert_eq!(validate_setupvars("IPV6_ADDRESS","2001:470:66:d5f:114b:a1b9:2a13:c7d9"), true);
        assert_eq!(validate_setupvars("PIHOLE_DNS_1","8.8.8.8"), true);
        assert_eq!(validate_setupvars("PIHOLE_DNS_2","8.8.4.4"), true);
        assert_eq!(validate_setupvars("PIHOLE_DOMAIN","lan"), true);
        assert_eq!(validate_setupvars("PIHOLE_INTERFACE","enp0s3"), true);
        assert_eq!(validate_setupvars("QUERY_LOGGING","true"), true);
        assert_eq!(validate_setupvars("WEBUIBOXEDLAYOUT","boxed"), true);
        assert_eq!(validate_setupvars("WEB_ENABLED","false"), true);
        // Nonsensical parameters
        assert_eq!(validate_setupvars("API_QUERY_LOG_SHOW","41"), false);
        assert_eq!(validate_setupvars("API_PRIVACY_MODE","off"), false);
        assert_eq!(validate_setupvars("DNS_BOGUS_PRIV","on"), false);
        assert_eq!(validate_setupvars("DNS_FQDN_REQUIRED","probably"), false);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING","disabled"), false);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING_DOMAIN","%%@)#"), false);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING_IP","192.1.1"), false);
        assert_eq!(validate_setupvars("CONDITIONAL_FORWARDING_REVERSE","in-addr.arpa.1.1.1"), false);
        assert_eq!(validate_setupvars("DHCP_ACTIVE","active"), false);
        assert_eq!(validate_setupvars("DHCP_END","2001:470:66:d5f:114b:a1b9:2a13:c7d9"), false);
        assert_eq!(validate_setupvars("DHCP_IPv6","ipv4"), false);
        assert_eq!(validate_setupvars("DHCP_LEASETIME","hours"), false);
        assert_eq!(validate_setupvars("DHCP_START","199199.1.0"), false);
        assert_eq!(validate_setupvars("DHCP_ROUTER","192.1681.1"), false);
        assert_eq!(validate_setupvars("DNSMASQ_LISTENING","dnsmasq"), false);
        assert_eq!(validate_setupvars("DNSSEC","enabled"), false);
        assert_eq!(validate_setupvars("INSTALL_WEB_SERVER","yes"), false);
        assert_eq!(validate_setupvars("INSTALL_WEB_INTERFACE","no"), false);
        assert_eq!(validate_setupvars("IPV4_ADDRESS","192.168.1.205"), false);
        assert_eq!(validate_setupvars("IPV6_ADDRESS","192.168.1.205"), false);
        assert_eq!(validate_setupvars("PIHOLE_DNS_1","www.pi-hole.net"), false);
        assert_eq!(validate_setupvars("PIHOLE_DNS_2","4.5"), false);
        assert_eq!(validate_setupvars("PIHOLE_DOMAIN","too many words"), false);
        assert_eq!(validate_setupvars("PIHOLE_INTERFACE","/dev/net/eth1"), false);
        assert_eq!(validate_setupvars("QUERY_LOGGING","disabled"), false);
        assert_eq!(validate_setupvars("WEBUIBOXEDLAYOUT","unboxed"), false);
        assert_eq!(validate_setupvars("WEB_ENABLED","457"), false);
        // Disabled / disallowed options - will report false.
        // Webpassword disallowed
        assert_eq!(validate_setupvars("WEBPASSWORD","B350486529B6022919491965A235157110B12437514315201184143ABBB37A14"), false);
        // Temperatureunit is not enabled
        assert_eq!(validate_setupvars("TEMPERATUREUNIT","K"), false);
    }

    #[test]
    fn test_validate_ftlconf() {
        // Acceptable paramaters
        assert_eq!(validate_ftlconf("SOCKET_LISTENING","localonly"), true);
        assert_eq!(validate_ftlconf("QUERY_DISPLAY","yes"), true);
        assert_eq!(validate_ftlconf("AAAA_QUERY_ANALYSIS","no"), true);
        assert_eq!(validate_ftlconf("RESOLVE_IPV6","yes"), true);
        assert_eq!(validate_ftlconf("RESOLVE_IPV4","no"), true);
        assert_eq!(validate_ftlconf("MAXDBDAYS","3"), true);
        assert_eq!(validate_ftlconf("DBINTERVAL","5.0"), true);
        assert_eq!(validate_ftlconf("DBFILE","/etc/pihole/FTL.conf"), true);
        assert_eq!(validate_ftlconf("MAXLOGAGE","8"), true);
        assert_eq!(validate_ftlconf("FTLPORT","64738"), true);
        assert_eq!(validate_ftlconf("PRIVACYLEVEL","2"), true);
        assert_eq!(validate_ftlconf("IGNORE_LOCALHOST","yes"), true);
        assert_eq!(validate_ftlconf("BLOCKINGMODE","NULL"), true);
        // Nonsensical parameters
        assert_eq!(validate_ftlconf("SOCKET_LISTENING","5"), false);
        assert_eq!(validate_ftlconf("QUERY_DISPLAY","true"), false);
        assert_eq!(validate_ftlconf("AAAA_QUERY_ANALYSIS",""), false);
        assert_eq!(validate_ftlconf("RESOLVE_IPV6","-1"), false);
        assert_eq!(validate_ftlconf("RESOLVE_IPV4","127.0.0.1"), false);
        assert_eq!(validate_ftlconf("MAXDBDAYS","nine"), false);
        assert_eq!(validate_ftlconf("DBINTERVAL","5.0.0"), false);
        assert_eq!(validate_ftlconf("DBFILE","http://www.pi-hole.net"), false);
        assert_eq!(validate_ftlconf("MAXLOGAGE","enabled"), false);
        assert_eq!(validate_ftlconf("FTLPORT","any"), false);
        assert_eq!(validate_ftlconf("PRIVACYLEVEL","high"), false);
        assert_eq!(validate_ftlconf("IGNORE_LOCALHOST","127.0.0.1"), false);
        assert_eq!(validate_ftlconf("BLOCKINGMODE","enabled"), false);
    }
}
