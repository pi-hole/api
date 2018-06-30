/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  DNS configuration information
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use rocket::State;
use setup_vars::read_setup_vars;
use util::{Error, Reply, reply_data};
use config::{Env};
use auth::User;
use routes::settings::convert::as_bool;

/// Get upstream DNS servers
fn get_upstream_dns(env: &State<Env>) -> Result<Vec<String>, Error> {
    let mut upstream_dns = Vec::new();
    for i in 1.. {
        let key = format!("PIHOLE_DNS_{}", i);
        let data = read_setup_vars(&key, &env)?;
        if let Some(ip) = data {
            upstream_dns.push(ip);
        } else {
            break
        }
    }

    Ok(upstream_dns)
}

/// Get DNS Configuration
#[get("/settings/dns")]
pub fn get_dns(env: State<Env>, _auth: User) -> Reply {
    let fqdn_required = read_setup_vars("DNS_FQDN_REQUIRED", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let bogus_priv = read_setup_vars("DNS_BOGUS_PRIV", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let dnssec = read_setup_vars("DNSSEC", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let cf_enabled = read_setup_vars("CONDITIONAL_FORWARDING", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let listening_type = read_setup_vars("DNSMASQ_LISTENING", &env)?
        .unwrap_or("single".to_owned());

    reply_data(json!({
        "upstream_dns": get_upstream_dns(&env)?,
        "options": {
          "fqdn_required": fqdn_required,
          "bogus_priv": bogus_priv,
          "dnssec": dnssec,
          "listening_type": listening_type
        },
        "conditional_forwarding": {
          "enabled": cf_enabled,
          "router_ip": read_setup_vars("CONDITIONAL_FORWARDING_IP", &env)?.unwrap_or_default(),
          "domain": read_setup_vars("CONDITIONAL_FORWARDING_DOMAIN", &env)?.unwrap_or_default(),
        }
    }))
}


#[cfg(test)]
mod test {
    use config::PiholeFile;
    use testing::TestBuilder;
    use rocket::http::Method;

    #[test]
    // Basic test for reported settings
    fn test_get_dns() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .method(Method::Get)
            .file(PiholeFile::SetupVars,
"DNSMASQ_LISTENING=all\nDNS_FQDN_REQUIRED=true\nDNS_BOGUS_PRIV=true\nDNSSEC=false\nPIHOLE_DNS_1=8.8.8.8\nPIHOLE_DNS_2=8.8.4.4\nCONDITIONAL_FORWARDING=true\nCONDITIONAL_FORWARDING_IP=192.168.1.1\nCONDITIONAL_FORWARDING_DOMAIN=hub\nCONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\n")
            .expect_json(
                json!({
                    "conditional_forwarding": {
                        "domain": "hub",
                        "enabled": true,
                        "router_ip": "192.168.1.1"
                    },
                    "options": {
                        "bogus_priv": true,
                        "dnssec": false,
                        "fqdn_required": true,
                        "listening_type": "all"
                    },
                    "upstream_dns": ["8.8.8.8", "8.8.4.4"]
                })
            )
            .test();
    }

    #[test]
    // Test that default settings are reported if not present
    fn test_get_dns_minimalsetup() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .method(Method::Get)
            .file(PiholeFile::SetupVars,
"WEBPASSWORD=\nPIHOLE_INTERFACE=eth0\nIPV4_ADDRESS=192.168.1.205/24\nIPV6_ADDRESS=\nPIHOLE_DNS_1=8.8.8.8\n")
            .expect_json(
                json!({
                    "conditional_forwarding": {
                        "domain": "",
                        "enabled": false,
                        "router_ip": ""
                    },
                    "options": {
                        "bogus_priv": false,
                        "dnssec": false,
                        "fqdn_required": false,
                        "listening_type": "single"
                    },
                    "upstream_dns": ["8.8.8.8"]
                })
            )
            .test();
    }

    #[test]
    // Test reporting settings from full setup file
    fn test_get_dns_fullsetup() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .method(Method::Get)
            .file(PiholeFile::SetupVars,
"WEBPASSWORD=841001982B38908BB1424B52990515474B77B05205B809304A10B21B03A93279\nAPI_QUERY_LOG_SHOW=all\nAPI_PRIVACY_MODE=false\nPIHOLE_INTERFACE=eth0\nIPV4_ADDRESS=192.168.1.205/24\nIPV6_ADDRESS=fd06:fb62:d251:9033:0:0:0:33\nQUERY_LOGGING=true\nINSTALL_WEB_SERVER=true\nINSTALL_WEB_INTERFACE=true\nLIGHTTPD_ENABLED=1\nTEMPERATUREUNIT=K\nWEBUIBOXEDLAYOUT=boxed\nDNSMASQ_LISTENING=all\nDNS_FQDN_REQUIRED=true\nDNS_BOGUS_PRIV=true\nDNSSEC=false\nPIHOLE_DNS_1=8.8.8.8\nPIHOLE_DNS_2=8.8.4.4\nCONDITIONAL_FORWARDING=true\nCONDITIONAL_FORWARDING_IP=192.168.1.1\nCONDITIONAL_FORWARDING_DOMAIN=hub\nCONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\nDHCP_START=192.168.1.201\nDHCP_END=192.168.1.251\nDHCP_ROUTER=192.168.1.1\nDHCP_LEASETIME=24\nPIHOLE_DOMAIN=lan\nDHCP_IPv6=false\nDHCP_ACTIVE=false\n")
            .expect_json(
                json!({
                    "conditional_forwarding": {
                        "domain": "hub",
                        "enabled": true,
                        "router_ip": "192.168.1.1"
                    },
                    "options": {
                        "bogus_priv": true,
                        "dnssec": false,
                        "fqdn_required": true,
                        "listening_type": "all"
                    },
                    "upstream_dns": ["8.8.8.8", "8.8.4.4"]
                })
            )
            .test();
    }

    #[test]
    // Specific test for case with multiple upstreams
    fn test_get_dns_multipleupstreams() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .file(PiholeFile::SetupVars,
"DNSMASQ_LISTENING=all\nDNS_FQDN_REQUIRED=true\nDNS_BOGUS_PRIV=true\nDNSSEC=false\nPIHOLE_DNS_1=8.8.8.8\nPIHOLE_DNS_2=7.7.7.7\nPIHOLE_DNS_3=6.6.6.6\nPIHOLE_DNS_4=5.5.5.5\nPIHOLE_DNS_5=22.22.22.22\nPIHOLE_DNS_6=31.31.31.31\nPIHOLE_DNS_7=40.40.40.40\nPIHOLE_DNS_8=1.0.0.0\nCONDITIONAL_FORWARDING=true\nCONDITIONAL_FORWARDING_IP=192.168.1.1\nCONDITIONAL_FORWARDING_DOMAIN=hub\nCONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\n")
            .expect_json(
                json!({
                    "conditional_forwarding": {
                        "domain": "hub",
                        "enabled": true,
                        "router_ip": "192.168.1.1"
                    },
                    "options": {
                        "bogus_priv": true,
                        "dnssec": false,
                        "fqdn_required": true,
                        "listening_type": "all"
                    },
                    "upstream_dns": ["8.8.8.8", "7.7.7.7", "6.6.6.6",
                                     "5.5.5.5", "22.22.22.22", "31.31.31.31",
                                     "40.40.40.40", "1.0.0.0"]
                })
            )
            .test();
    }
}
