/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for reading domain lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::Config;
use dns::list::{List, get_list};
use rocket::State;
use util;

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(config: State<Config>) -> util::Reply {
    get_list(List::Whitelist, &config)
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(config: State<Config>) -> util::Reply {
    get_list(List::Blacklist, &config)
}

/// Get the Wildcard list domains
#[get("/dns/wildlist")]
pub fn get_wildlist(config: State<Config>) -> util::Reply {
    get_list(List::Wildlist, &config)
}
