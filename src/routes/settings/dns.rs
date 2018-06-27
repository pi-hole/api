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
use util::{Reply, reply_data};
use config::{Env};
use auth::User;
use std::str::FromStr;

/// Get DNS Configuration & Upstream servers
#[get("/settings/dns")]
pub fn dns(env: State<Env>, _auth: User) -> Reply {

    // Convert other data types returned as strings.
    let fqdn_required : bool = FromStr::from_str(&read_setup_vars("DNS_FQDN_REQUIRED", &env)?.unwrap_or_default()).unwrap_or_default();
    let bogus_priv : bool = FromStr::from_str(&read_setup_vars("DNS_BOGUS_PRIV", &env)?.unwrap_or_default()).unwrap_or_default();
    let dnssec : bool = FromStr::from_str(&read_setup_vars("DNSSEC", &env)?.unwrap_or_default()).unwrap_or_default();
    let cf_enabled : bool = FromStr::from_str(&read_setup_vars("CONDITIONAL_FORWARDING", &env)?.unwrap_or_default()).unwrap_or_default();

    return reply_data(json!({
        "upstream": {
          "dns_1": read_setup_vars("PIHOLE_DNS_1", &env)?.unwrap_or_default(),
          "dns_2": read_setup_vars("PIHOLE_DNS_2", &env)?.unwrap_or_default()
        },
        "options": {
          "fqdn_required": fqdn_required,
          "bogus_priv": bogus_priv,
          "dnssec": dnssec,
          "dnsmasq_listening": read_setup_vars("DNSMASQ_LISTENING", &env)?.unwrap_or_default()
        },
        "conditional_formatting": {
          "enabled": cf_enabled,
          "ip": read_setup_vars("CONDITIONAL_FORWARDING_IP", &env)?.unwrap_or_default(),
          "domain": read_setup_vars("CONDITIONAL_FORWARDING_DOMAIN", &env)?.unwrap_or_default(),
          "reverse": read_setup_vars("CONDITIONAL_FORWARDING_REVERSE", &env)?.unwrap_or_default()
        }
    }));
}
