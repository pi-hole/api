/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for adding domains to lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::Config;
use dns::common::reload_gravity;
use dns::list::{add_list, List, try_remove_list};
use rocket::State;
use rocket_contrib::Json;
use util;

/// Represents an API input containing a domain
#[derive(Deserialize)]
pub struct DomainInput {
    domain: String
}

/// Add a domain to the whitelist
#[post("/dns/whitelist", data = "<domain_input>")]
pub fn add_whitelist(config: State<Config>, domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the whitelist and remove it from the other lists
    add_list(List::Whitelist, domain, &config)?;
    try_remove_list(List::Blacklist, domain, &config)?;
    try_remove_list(List::Wildlist, domain, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    reload_gravity(List::Whitelist)?;
    util::reply_success()
}

/// Add a domain to the blacklist
#[post("/dns/blacklist", data = "<domain_input>")]
pub fn add_blacklist(config: State<Config>, domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the blacklist and remove it from the other lists
    add_list(List::Blacklist, domain, &config)?;
    try_remove_list(List::Whitelist, domain, &config)?;
    try_remove_list(List::Wildlist, domain, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    reload_gravity(List::Blacklist)?;
    util::reply_success()
}

/// Add a domain to the wildcard list
#[post("/dns/wildlist", data = "<domain_input>")]
pub fn add_wildlist(config: State<Config>, domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We only need to add it to the wildcard list (this is the same functionality as list.sh)
    add_list(List::Wildlist, domain, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    reload_gravity(List::Wildlist)?;
    util::reply_success()
}
