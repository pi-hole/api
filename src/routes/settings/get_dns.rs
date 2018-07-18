// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// DNS Configuration Settings
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use env::Env;
use rocket::State;
use routes::settings::common::as_bool;
use settings::{ConfigEntry, SetupVarsEntry};
use util::{reply_data, Error, Reply};

/// Get upstream DNS servers
fn get_upstream_dns(env: &State<Env>) -> Result<Vec<String>, Error> {
    let mut upstream_dns = Vec::new();

    for num in 1.. {
        let ip = SetupVarsEntry::PiholeDns(num).read(&env)?;

        if !ip.is_empty() {
            upstream_dns.push(ip);
        } else {
            break;
        }
    }

    Ok(upstream_dns)
}

/// Get DNS Configuration
#[get("/settings/dns")]
pub fn get_dns(env: State<Env>, _auth: User) -> Reply {
    let fqdn_required = as_bool(&SetupVarsEntry::DnsFqdnRequired.read(&env)?);
    let bogus_priv = as_bool(&SetupVarsEntry::DnsBogusPriv.read(&env)?);
    let dnssec = as_bool(&SetupVarsEntry::Dnssec.read(&env)?);
    let cf_enabled = as_bool(&SetupVarsEntry::ConditionalForwarding.read(&env)?);
    let listening_type = &SetupVarsEntry::DnsmasqListening.read(&env)?;

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
            "router_ip": SetupVarsEntry::ConditionalForwardingIp.read(&env)?,
            "domain": SetupVarsEntry::ConditionalForwardingDomain.read(&env)?,
        }
    }))
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use testing::TestBuilder;

    /// Basic test for reported settings
    #[test]
    fn test_get_dns_multiple_upstreams() {
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
                 CONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\n"
            )
            .expect_json(json!({
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
            }))
            .test();
    }

    /// Test that default settings are reported if not present
    #[test]
    fn test_get_dns_minimal_setup() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .file(PiholeFile::SetupVars, "")
            .expect_json(json!({
                "conditional_forwarding": {
                    "domain": "",
                    "enabled": false,
                    "router_ip": ""
                },
                "options": {
                    "bogus_priv": true,
                    "dnssec": false,
                    "fqdn_required": true,
                    "listening_type": "single"
                },
                "upstream_dns": []
            }))
            .test();
    }
}
