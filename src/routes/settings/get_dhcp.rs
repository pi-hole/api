// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// DHCP Configuration Settings
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use config::Env;
use rocket::State;
use routes::settings::common::as_bool;
use settings::{read_setup_vars, SetupVarsEntry};
use util::{reply_data, Reply};

/// Get DHCP configuration
#[get("/settings/dhcp")]
pub fn get_dhcp(env: State<Env>, _auth: User) -> Reply {
    let dhcp_active = read_setup_vars(SetupVarsEntry::DhcpActive, &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let ipv6_support = read_setup_vars(SetupVarsEntry::DhcpIpv6, &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let lease_time = read_setup_vars(SetupVarsEntry::DhcpLeasetime, &env)?
        .unwrap_or_default()
        .parse::<i32>()
        .unwrap_or(24);

    reply_data(json!({
      "active": dhcp_active,
      "ip_start": read_setup_vars(SetupVarsEntry::DhcpStart, &env)?.unwrap_or_default(),
      "ip_end": read_setup_vars(SetupVarsEntry::DhcpEnd, &env)?.unwrap_or_default(),
      "router_ip": read_setup_vars(SetupVarsEntry::DhcpRouter, &env)?.unwrap_or_default(),
      "lease_time": lease_time,
      "domain": read_setup_vars(SetupVarsEntry::PiholeDomain, &env)?.unwrap_or_default(),
      "ipv6_support": ipv6_support
    }))
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use testing::TestBuilder;

    /// Basic test for reported settings
    #[test]
    fn test_get_dhcp() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dhcp")
            .file(
                PiholeFile::SetupVars,
                "DHCP_START=192.168.1.201\n\
                 DHCP_END=192.168.1.251\n\
                 DHCP_ROUTER=192.168.1.1\n\
                 DHCP_LEASETIME=24\n\
                 PIHOLE_DOMAIN=lan\n\
                 DHCP_IPv6=false\n\
                 DHCP_ACTIVE=false\n"
            )
            .expect_json(json!({
                "active": false,
                "ip_start": "192.168.1.201",
                "ip_end": "192.168.1.251",
                "router_ip": "192.168.1.1",
                "lease_time": 24,
                "domain": "lan",
                "ipv6_support": false,
            }))
            .test();
    }

    /// Test that default settings are reported if not present
    #[test]
    fn test_get_dhcp_minimal_setup() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dhcp")
            .file(PiholeFile::SetupVars, "")
            .expect_json(json!({
                "active": false,
                "ip_start": "",
                "ip_end": "",
                "router_ip": "",
                "lease_time": 24,
                "domain": "",
                "ipv6_support": false,
            }))
            .test();
    }
}
