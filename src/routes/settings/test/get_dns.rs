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
#[get("/settings/get_dns")]
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
    fn test_get_dns() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/get_dns")
            .method(Method::Get)
            .file(PiholeFile::SetupVars, 
                "WEBPASSWORD=841001982B38908BB1424B52990515474B77B05205B809304A10B21B03A93279\n
                API_QUERY_LOG_SHOW=all\n
                API_PRIVACY_MODE=false\n
                PIHOLE_INTERFACE=enp0s3\n
                IPV4_ADDRESS=192.168.1.205/24\n
                IPV6_ADDRESS=fd06:fb62:d251:9033:0:0:0:33\n
                QUERY_LOGGING=true\n
                INSTALL_WEB_SERVER=true\n
                INSTALL_WEB_INTERFACE=true\n
                LIGHTTPD_ENABLED=1\n
                TEMPERATUREUNIT=K\n
                WEBUIBOXEDLAYOUT=boxed\n
                DNSMASQ_LISTENING=all\n
                DNS_FQDN_REQUIRED=true\n
                DNS_BOGUS_PRIV=true\n
                DNSSEC=false\n
                PIHOLE_DNS_1=8.8.8.8\n
                PIHOLE_DNS_2=8.8.4.4\n
                CONDITIONAL_FORWARDING=true\n
                CONDITIONAL_FORWARDING_IP=192.168.1.1\n
                CONDITIONAL_FORWARDING_DOMAIN=hub\n
                CONDITIONAL_FORWARDING_REVERSE=1.168.192.in-addr.arpa\n
                DHCP_START=192.168.1.201\n
                DHCP_END=192.168.1.251\n
                DHCP_ROUTER=192.168.1.1\n
                DHCP_LEASETIME=24\n
                PIHOLE_DOMAIN=lan\n
                DHCP_IPv6=false\n
                DHCP_ACTIVE=false\n")
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
}
