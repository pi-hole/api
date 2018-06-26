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
pub fn network(env: State<Env>, _auth: User) -> Reply {
    let ipv4_full = read_setup_vars("IPV4_ADDRESS", &env)?.unwrap_or_default();
    let ipv4_address: Vec<&str> = ipv4_full.split("/").collect();
    return reply_data(json!({
        "interface": read_setup_vars("PIHOLE_INTERFACE", &env)?.unwrap_or_default(),
        "ipv4_address": &ipv4_address[0],
        "ipv6_address": read_setup_vars("IPV6_ADDRESS", &env)?.unwrap_or_default(),
        "hostname": get_hostname().unwrap_or("unknown".to_owned())
    }));
}
