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
