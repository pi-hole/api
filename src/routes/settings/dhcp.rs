/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  DHCP configuration information
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use rocket::State;
use setup_vars::read_setup_vars;
use util::{Reply, reply_data};
use config::{Env};
use auth::User;

/// Get Pi-hole local network information
#[get("/settings/dhcp")]
pub fn dhcp(env: State<Env>, _auth: User) -> Reply {
    return reply_data(json!({
      "active": read_setup_vars("DHCP_ACTIVE", &env)?.unwrap_or_default(),
      "ip_start": read_setup_vars("DHCP_START", &env)?.unwrap_or_default(),
      "ip_end": read_setup_vars("DHCP_END", &env)?.unwrap_or_default(),
      "router_ip": read_setup_vars("DHCP_ROUTER", &env)?.unwrap_or_default(),
      "lease_time": read_setup_vars("DHCP_LEASETIME", &env)?.unwrap_or_default(),
      "domain": read_setup_vars("PIHOLE_DOMAIN", &env)?.unwrap_or_default(),
      "ipv6_support": read_setup_vars("DHCP_IPv6", &env)?.unwrap_or_default()
    }));
}
