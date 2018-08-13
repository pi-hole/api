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
use rocket_contrib::Json;
use routes::settings::common::as_bool;
use routes::settings::common::restart_dns;
use settings::{generate_dnsmasq_config, ConfigEntry, SetupVarsEntry};
use util::{reply_data, reply_success, Error, ErrorKind, Reply};

#[derive(Serialize, Deserialize)]
pub struct DnsSettings {
    upstream_dns: Vec<String>,
    options: DnsOptions,
    conditional_forwarding: DnsConditionalForwarding
}

impl DnsSettings {
    /// Check if all the DNS settings are valid
    fn is_valid(&self) -> bool {
        self.upstream_dns
            .iter()
            .all(|dns| SetupVarsEntry::PiholeDns(0).is_valid(dns))
            && self.options.is_valid() && self.conditional_forwarding.is_valid()
    }
}

#[derive(Serialize, Deserialize)]
pub struct DnsOptions {
    fqdn_required: bool,
    bogus_priv: bool,
    dnssec: bool,
    listening_type: String
}

impl DnsOptions {
    /// Check if the DNS settings are valid
    fn is_valid(&self) -> bool {
        // The boolean values are all valid because they were parsed into booleans
        // already
        SetupVarsEntry::DnsmasqListening.is_valid(&self.listening_type)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DnsConditionalForwarding {
    enabled: bool,
    router_ip: String,
    domain: String
}

impl DnsConditionalForwarding {
    /// Check if the conditional forwarding options are valid
    fn is_valid(&self) -> bool {
        // `enabled` is already known to be valid because it was already parsed into
        // a boolean
        SetupVarsEntry::DhcpRouter.is_valid(&self.router_ip)
            && SetupVarsEntry::ConditionalForwardingDomain.is_valid(&self.domain)
    }
}

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
    let dns_settings = DnsSettings {
        upstream_dns: get_upstream_dns(&env)?,
        options: DnsOptions {
            fqdn_required: as_bool(&SetupVarsEntry::DnsFqdnRequired.read(&env)?),
            bogus_priv: as_bool(&SetupVarsEntry::DnsBogusPriv.read(&env)?),
            dnssec: as_bool(&SetupVarsEntry::Dnssec.read(&env)?),
            listening_type: SetupVarsEntry::DnsmasqListening.read(&env)?
        },
        conditional_forwarding: DnsConditionalForwarding {
            enabled: as_bool(&SetupVarsEntry::ConditionalForwarding.read(&env)?),
            router_ip: SetupVarsEntry::ConditionalForwardingIp.read(&env)?,
            domain: SetupVarsEntry::ConditionalForwardingDomain.read(&env)?
        }
    };

    reply_data(dns_settings)
}

/// Update DNS Configuration
#[put("/settings/dns", data = "<data>")]
pub fn put_dns(env: State<Env>, _auth: User, data: Json<DnsSettings>) -> Reply {
    let settings: DnsSettings = data.into_inner();

    if !settings.is_valid() {
        return Err(ErrorKind::InvalidSettingValue.into());
    }

    // Delete previous upstream DNS entries
    SetupVarsEntry::delete_upstream_dns(&env)?;

    // Add new upstream DNS
    for (i, dns) in settings.upstream_dns.into_iter().enumerate() {
        SetupVarsEntry::PiholeDns(i + 1).write(&dns, &env)?;
    }

    // Write DNS settings to SetupVars
    SetupVarsEntry::DnsFqdnRequired.write(&settings.options.fqdn_required.to_string(), &env)?;
    SetupVarsEntry::DnsBogusPriv.write(&settings.options.bogus_priv.to_string(), &env)?;
    SetupVarsEntry::Dnssec.write(&settings.options.dnssec.to_string(), &env)?;
    SetupVarsEntry::DnsmasqListening.write(&settings.options.listening_type, &env)?;

    if settings.conditional_forwarding.enabled {
        let address_segments: Vec<&str> = settings
            .conditional_forwarding
            .router_ip
            .split(".")
            .take(3)
            .collect();
        let reverse_address = format!(
            "{}.{}.{}.in-addr.arpa",
            address_segments[2], address_segments[1], address_segments[0]
        );

        SetupVarsEntry::ConditionalForwarding.write("true", &env)?;
        SetupVarsEntry::ConditionalForwardingReverse.write(&reverse_address, &env)?;
        SetupVarsEntry::ConditionalForwardingIp
            .write(&settings.conditional_forwarding.router_ip, &env)?;
        SetupVarsEntry::ConditionalForwardingDomain
            .write(&settings.conditional_forwarding.domain, &env)?;
    } else {
        SetupVarsEntry::ConditionalForwarding.write("false", &env)?;
        SetupVarsEntry::ConditionalForwardingReverse.delete(&env)?;
        SetupVarsEntry::ConditionalForwardingIp.delete(&env)?;
        SetupVarsEntry::ConditionalForwardingDomain.delete(&env)?;
    }

    generate_dnsmasq_config(&env)?;
    restart_dns(&env)?;
    reply_success()
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use testing::TestBuilder;
    use rocket::http::Method;

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

    /// Test updating with new settings
    #[test]
    fn test_put_dns() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dns")
            .method(Method::Put)
            .file_expect(
                PiholeFile::SetupVars,
                "",
                "PIHOLE_DNS_1=8.8.8.8\n\
                PIHOLE_DNS_2=8.8.4.4\n\
                DNS_FQDN_REQUIRED=true\n\
                DNS_BOGUS_PRIV=true\n\
                DNSSEC=true\n\
                DNSMASQ_LISTENING=local\n\
                CONDITIONAL_FORWARDING=true\n\
                CONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\n\
                CONDITIONAL_FORWARDING_IP=192.168.1.1\n\
                CONDITIONAL_FORWARDING_DOMAIN=local\n"
            )
            .file_expect(
                PiholeFile::DnsmasqConfig,
                "",
                "################################################################\n\
                    #       THIS FILE IS AUTOMATICALLY GENERATED BY PI-HOLE.       #\n\
                    #          ANY CHANGES MADE TO THIS FILE WILL BE LOST.         #\n\
                    #                                                              #\n\
                    #  NEW CONFIG SETTINGS MUST BE MADE IN A SEPARATE CONFIG FILE  #\n\
                    #                OR IN /etc/dnsmasq.conf                       #\n\
                    ################################################################\n\
                    \n\
                    localise-queries\n\
                    server=8.8.8.8\n\
                    server=8.8.4.4\n\
                    addn-hosts=/etc/pihole/gravity.list\n\
                    addn-hosts=/etc/pihole/black.list\n\
                    addn-hosts=/etc/pihole/local.list\n\
                    domain-needed\n\
                    bogus-priv\n\
                    dnssec\n\
                    trust-anchor=.,19036,8,2,49AAC11D7B6F6446702E54A1607371607A1A41855200FD2CE1CDDE32F24E8FB5\n\
                    trust-anchor=.,20326,8,2,E06D44B80B8F1D39A95C0B0D7C65D08458E880409BBC683457104237C7F8EC8D\n\
                    local-service\n\
                    server=/local/192.168.1.1\n\
                    server=/1.168.192.in-addr.arpa/192.168.1.1\n"
            )
            .body(json!({
                "upstream_dns": [
                    "8.8.8.8", "8.8.4.4"
                ],
                "conditional_forwarding": {
                    "domain": "local",
                    "enabled": true,
                    "router_ip": "192.168.1.1"
                },
                "options": {
                    "bogus_priv": true,
                    "dnssec": true,
                    "fqdn_required": true,
                    "listening_type": "local"
                }
            }))
            .expect_json(json!({
                "status": "success"
            }))
            .test();
    }
}
