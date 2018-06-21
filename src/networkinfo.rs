/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Local network information
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

extern crate hostname;

use setup_vars;
use util;
use config::{Env, Config};

const CONFIG_LOCATION: &'static str = "/etc/pihole/API.toml";

/// Get Pi-hole host name (not contained in setupvars.conf)
fn gethost() -> String {
    match hostname::get_hostname() {
        None => {
            return "unknown".to_owned();
        }
        Some(h) => {
            return h;
        }
    }   
}

/// Get an entry from setupvars.conf, handle errors or nulls
fn getsetupentry(searchentry: String) -> String {
    let config = Config::parse(CONFIG_LOCATION).unwrap();
    let env = Env::Production(config);
    match setup_vars::read_setup_vars(&searchentry, &env) {
        Ok(i) => {
            match i {
                Some(j) => { return j; }
                None => { return "".to_owned(); }
            }
        }
        Err(_) => {
            return "error".to_owned();
        }
    }
}

/// Get Pi-hole IPv4 address
fn getipv4() -> String {
    getsetupentry("IPV4_ADDRESS".to_owned())
}

/// Get Pi-hole IPv6 address
fn getipv6() -> String {
    getsetupentry("IPV6_ADDRESS".to_owned())
}

/// Get Pi-hole hostname
fn getinterface() -> String {
    getsetupentry("PIHOLE_INTERFACE".to_owned())
}

/// Get Pi-hole local network information
#[get("/networkinfo")]
pub fn networkinfo() -> util::Reply {
    return util::reply_data(json!({
        "netinterface": getinterface(),
        "netipv4": getipv4(),
        "netipv6": getipv6(),
        "nethost": gethost()
    }));  
}
