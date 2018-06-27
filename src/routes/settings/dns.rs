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

/// Get Pi-hole local network information
#[get("/settings/dns")]
pub fn dns(env: State<Env>, _auth: User) -> Reply {
    return reply_data(json!({
        "upstream": {
          "dns_1": read_setup_vars("PIHOLE_DNS_1", &env)?.unwrap_or_default(),
          "dns_2": read_setup_vars("PIHOLE_DNS_2", &env)?.unwrap_or_default()
        },
        "options": {
          "fqdn_required": read_setup_vars("DNS_FQDN_REQUIRED", &env)?.unwrap_or_default(),
          "bogus_priv": read_setup_vars("DNS_BOGUS_PRIV", &env)?.unwrap_or_default(),
          "dnssec": read_setup_vars("DNSSEC", &env)?.unwrap_or_default(),
          "dnsmasq_listening": read_setup_vars("DNSMASQ_LISTENING", &env)?.unwrap_or_default()
        },
        "conditional_formatting": {
          "enabled": read_setup_vars("CONDITIONAL_FORWARDING", &env)?.unwrap_or_default(),
          "ip": read_setup_vars("CONDITIONAL_FORWARDING_IP", &env)?.unwrap_or_default(),
          "domain": read_setup_vars("CONDITIONAL_FORWARDING_DOMAIN", &env)?.unwrap_or_default(),
          "reverse": read_setup_vars("CONDITIONAL_FORWARDING_REVERSE", &env)?.unwrap_or_default()
        }
    }));
}
