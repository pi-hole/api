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
use env::Env;
use rocket::State;
use routes::settings::common::as_bool;
use settings::{ConfigEntry, SetupVarsEntry};
use util::{reply_data, Reply};

/// Get DHCP configuration
#[get("/settings/dhcp")]
pub fn get_dhcp(env: State<Env>, _auth: User) -> Reply {
    let dhcp_active = as_bool(&SetupVarsEntry::DhcpActive.read(&env)?);
    let ipv6_support = as_bool(&SetupVarsEntry::DhcpIpv6.read(&env)?);
    let lease_time: i32 = SetupVarsEntry::DhcpLeasetime.read(&env)?
        .parse()
        .unwrap_or(SetupVarsEntry::DhcpLeasetime.get_default().parse().unwrap());

    reply_data(json!({
      "active": dhcp_active,
      "ip_start": SetupVarsEntry::DhcpStart.read(&env)?,
      "ip_end": SetupVarsEntry::DhcpEnd.read(&env)?,
      "router_ip": SetupVarsEntry::DhcpRouter.read(&env)?,
      "lease_time": lease_time,
      "domain": SetupVarsEntry::PiholeDomain.read(&env)?,
      "ipv6_support": ipv6_support
    }))
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
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
