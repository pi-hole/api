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
    fn test_get_dns_multipleupstreams() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .file(
                PiholeFile::SetupVars,
                "DNSMASQ_LISTENING=all\n\
                DNS_FQDN_REQUIRED=true\n\
                DNS_BOGUS_PRIV=true\n\
                DNSSEC=false\n\
                PIHOLE_DNS_1=8.8.8.8\n\
                PIHOLE_DNS_2=7.7.7.7\n\
                PIHOLE_DNS_3=6.6.6.6\n\
                PIHOLE_DNS_4=5.5.5.5\n\
                PIHOLE_DNS_5=22.22.22.22\n\
                PIHOLE_DNS_6=31.31.31.31\n\
                PIHOLE_DNS_7=40.40.40.40\n\
                PIHOLE_DNS_8=1.0.0.0\n\
                CONDITIONAL_FORWARDING=true\n\
                CONDITIONAL_FORWARDING_IP=192.168.1.1\n\
                CONDITIONAL_FORWARDING_DOMAIN=hub\n\
                CONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\n")
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
                    "upstream_dns": [
                        "8.8.8.8",
                        "7.7.7.7",
                        "6.6.6.6",
                        "5.5.5.5",
                        "22.22.22.22",
                        "31.31.31.31",
                        "40.40.40.40",
                        "1.0.0.0"
                    ]
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
            .file(PiholeFile::SetupVars, "")
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
                    "upstream_dns": []
                })
            )
            .test();
    }
}
