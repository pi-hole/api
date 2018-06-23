/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Local hostname
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use rocket::State;
use setup_vars::read_setup_vars;
use util;
use config::{Env};
use hostname;

/// Get Pi-hole host name
fn get_hostname() -> String {
    hostname::get_hostname()
        .unwrap_or("unknown".to_owned())
}

/// Get Pi-hole local network information
#[get("/settings/network")]
pub fn network(env: State<Env>) -> util::Reply {
    return util::reply_data(json!({
        "interface": read_setup_vars(&"PIHOLE_INTERFACE", &env)?.unwrap_or_default(),
        "ipv4_address": read_setup_vars(&"IPV4_ADDRESS", &env)?.unwrap_or_default(),
        "ipv6_address": read_setup_vars(&"IPV6_ADDRESS", &env)?.unwrap_or_default(),
        "hostname": get_hostname()
    }));  
}

