/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  DNS configuration information
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::collections::HashMap;
use rocket::State;
use setup_vars::read_setup_vars;
use util::{Reply, reply_data};
use config::{Env};
use auth::User;

/// Convert booleans returned as strings.
fn as_bool(t: &str) -> bool {
  match t.to_lowercase().as_str() {
    "true" | "1" => true,
    "false" | "0" => false,
    _ => false
  }
}

/// Get DNS Configuration & Upstream servers
#[get("/settings/dns")]
pub fn dns(env: State<Env>, _auth: User) -> Reply {
    // Get upstream DNS servers - number may vary
    let mut server_list = HashMap::new();
    let dns = "dns_";
    let pidns = "PIHOLE_DNS_";
    let mut dns_counter = 1;
    while dns_counter > 0 {  
        // setup search and output strings
        let mut dns_number = dns.to_owned();
        let mut pidns_number = pidns.to_owned();
        dns_number.push_str(&dns_counter.to_string());
        pidns_number.push_str(&dns_counter.to_string());
        let upstreamdns : String = read_setup_vars(&pidns_number, &env)?.unwrap_or_default();
        if upstreamdns != "" {
            server_list.insert(dns_number, upstreamdns);
            dns_counter += 1 ;
        }
        else {
            dns_counter = -1 
        }
    }  
    let fqdn_required: bool = read_setup_vars("DNS_FQDN_REQUIRED", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let bogus_priv: bool = read_setup_vars("DNS_BOGUS_PRIV", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let dnssec: bool = read_setup_vars("DNSSEC", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let cf_enabled: bool = read_setup_vars("CONDITIONAL_FORWARDING", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let listening_type = read_setup_vars("DNSMASQ_LISTENING", &env)?.unwrap_or("single".to_owned());

    return reply_data(json!({
        "upstream_dns": 
           server_list,
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
    }));
}
