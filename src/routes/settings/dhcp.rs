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
use routes::settings::convert::as_bool;

/// Get DHCP configuration
#[get("/settings/dhcp")]
pub fn dhcp(env: State<Env>, _auth: User) -> Reply {
    let dhcp_active: bool = read_setup_vars("DHCP_ACTIVE", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let ipv6_support: bool = read_setup_vars("DHCP_IPv6", &env)?
        .map(|s| as_bool(&s))
        .unwrap_or(false);
    let lease_time: i32 = read_setup_vars("DHCP_LEASETIME", &env)?
        .unwrap_or_default()
        .parse::<i32>()
        .unwrap_or_default();

    return reply_data(json!({
      "active": dhcp_active,
      "ip_start": read_setup_vars("DHCP_START", &env)?.unwrap_or_default(),
      "ip_end": read_setup_vars("DHCP_END", &env)?.unwrap_or_default(),
      "router_ip": read_setup_vars("DHCP_ROUTER", &env)?.unwrap_or_default(),
      "lease_time": lease_time,
      "domain": read_setup_vars("PIHOLE_DOMAIN", &env)?.unwrap_or_default(),
      "ipv6_support": ipv6_support
    }));
}

