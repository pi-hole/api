/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Local network information
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use rocket::State;
use setup_vars::read_setup_vars;
use util::{Reply, reply_data};
use config::{Env};
use hostname::get_hostname;
use auth::User;

/// Get Pi-hole local network information
#[get("/settings/network")]
pub fn get_network(env: State<Env>, _auth: User) -> Reply {
    let ipv4_full = read_setup_vars("IPV4_ADDRESS", &env)?.unwrap_or_default();
    let ipv4_address: Vec<&str> = ipv4_full.split("/").collect();
    let ipv6_full = read_setup_vars("IPV6_ADDRESS", &env)?.unwrap_or_default();
    let ipv6_address: Vec<&str> = ipv6_full.split("/").collect();

    reply_data(json!({
        "interface": read_setup_vars("PIHOLE_INTERFACE", &env)?.unwrap_or_default(),
        "ipv4_address": ipv4_address[0],
        "ipv6_address": ipv6_address[0],
        "hostname": get_hostname().unwrap_or("unknown".to_owned())
    }))
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use testing::TestBuilder;
    use hostname::get_hostname;

    #[test]
    // Basic test for reported settings
    fn test_get_network() {
        let currenthost = get_hostname().unwrap_or("unknown".to_owned());
        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(PiholeFile::SetupVars,"IPV4_ADDRESS=192.168.1.205/24\nIPV6_ADDRESS=fd06:fb62:d251:9033:0:0:0:33\nPIHOLE_INTERFACE=eth0\n")
            .expect_json(
                json!({
                    "interface": "eth0",
                    "ipv4_address": "192.168.1.205",
                    "ipv6_address": "fd06:fb62:d251:9033:0:0:0:33",
                    "hostname": currenthost
                })
            )
            .test();
    }

    #[test]
    // Test for common configuration of ipv4 only (no ipv6)
    fn test_get_network_ipv4only() {
        let currenthost = get_hostname().unwrap_or("unknown".to_owned());
        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(PiholeFile::SetupVars,"IPV4_ADDRESS=192.168.1.205/24\nIPV6_ADDRESS=\nPIHOLE_INTERFACE=eth0\n")
            .expect_json(
                json!({
                    "interface": "eth0",
                    "ipv4_address": "192.168.1.205",
                    "ipv6_address": "",
                    "hostname": currenthost
                })
            )
            .test();
    }

  #[test]
    // Test reporting settings from full setup file
    fn test_get_network_fullsetup() {
        let currenthost = get_hostname().unwrap_or("unknown".to_owned());
        TestBuilder::new()
            .endpoint("/admin/api/settings/network")
            .file(PiholeFile::SetupVars,
"WEBPASSWORD=841001982B38908BB1424B52990515474B77B05205B809304A10B21B03A93279\n API_QUERY_LOG_SHOW=all\nAPI_PRIVACY_MODE=false\nPIHOLE_INTERFACE=eth0\nIPV4_ADDRESS=192.168.1.205/24\nIPV6_ADDRESS=fd06:fb62:d251:9033:0:0:0:33\nQUERY_LOGGING=true\nINSTALL_WEB_SERVER=true\nINSTALL_WEB_INTERFACE=true\nLIGHTTPD_ENABLED=1\nTEMPERATUREUNIT=K\nWEBUIBOXEDLAYOUT=boxed\nDNSMASQ_LISTENING=all\nDNS_FQDN_REQUIRED=true\nDNS_BOGUS_PRIV=true\nDNSSEC=false\nPIHOLE_DNS_1=8.8.8.8\nPIHOLE_DNS_2=8.8.4.4\nCONDITIONAL_FORWARDING=true\nCONDITIONAL_FORWARDING_IP=192.168.1.1\nCONDITIONAL_FORWARDING_DOMAIN=hub\nCONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\nDHCP_START=192.168.1.201\nDHCP_END=192.168.1.251\nDHCP_ROUTER=192.168.1.1\nDHCP_LEASETIME=24\nPIHOLE_DOMAIN=lan\nDHCP_IPv6=false\nDHCP_ACTIVE=false\n")
            .expect_json(
                json!({
                    "interface": "eth0",
                    "ipv4_address": "192.168.1.205",
                    "ipv6_address": "fd06:fb62:d251:9033:0:0:0:33",
                    "hostname": currenthost
                })
            )
            .test();
    }
}
