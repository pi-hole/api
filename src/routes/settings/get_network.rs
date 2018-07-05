// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Local Network Settings
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use config::Env;
use hostname::get_hostname;
use rocket::State;
use setup_vars::read_setup_vars;
use util::{reply_data, Reply};
use config_files::SetupVarsEntry::*;

/// Get Pi-hole local network information
#[get("/settings/network")]
pub fn get_network(env: State<Env>, _auth: User) -> Reply {
    let ipv4_full = read_setup_vars(Ipv4Address, &env)?.unwrap_or_default();
    let ipv4_address: Vec<&str> = ipv4_full.split("/").collect();
    let ipv6_full = read_setup_vars(Ipv6Address, &env)?.unwrap_or_default();
    let ipv6_address: Vec<&str> = ipv6_full.split("/").collect();

    reply_data(json!({
        "interface": read_setup_vars(PiholeInterface, &env)?.unwrap_or_default(),
        "ipv4_address": ipv4_address[0],
        "ipv6_address": ipv6_address[0],
        "hostname": get_hostname().unwrap_or("unknown".to_owned())
    }))
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use hostname::get_hostname;
    use testing::TestBuilder;

    /// Basic test for reported settings
    #[test]
    fn test_get_network() {
        let current_host = get_hostname().unwrap_or("unknown".to_owned());

        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(
                PiholeFile::SetupVars,
                "IPV4_ADDRESS=192.168.1.205/24\n\
                 IPV6_ADDRESS=fd06:fb62:d251:9033:0:0:0:33\n\
                 PIHOLE_INTERFACE=eth0\n"
            )
            .expect_json(json!({
                "interface": "eth0",
                "ipv4_address": "192.168.1.205",
                "ipv6_address": "fd06:fb62:d251:9033:0:0:0:33",
                "hostname": current_host
            }))
            .test();
    }

    /// Test for common configuration of ipv4 only (no ipv6)
    #[test]
    fn test_get_network_ipv4only() {
        let current_host = get_hostname().unwrap_or("unknown".to_owned());

        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(
                PiholeFile::SetupVars,
                "IPV4_ADDRESS=192.168.1.205/24\n\
                 IPV6_ADDRESS=\n\
                 PIHOLE_INTERFACE=eth0\n"
            )
            .expect_json(json!({
                "interface": "eth0",
                "ipv4_address": "192.168.1.205",
                "ipv6_address": "",
                "hostname": current_host
            }))
            .test();
    }
}
