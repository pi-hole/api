/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for removing domains from lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::Config;
use dns::common::reload_gravity;
use dns::list::{List, remove_list};
use rocket::State;
use util;

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(config: State<Config>, domain: String) -> util::Reply {
    remove_list(List::Whitelist, &domain, &config)?;
    reload_gravity(List::Whitelist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(config: State<Config>, domain: String) -> util::Reply {
    remove_list(List::Blacklist, &domain, &config)?;
    reload_gravity(List::Blacklist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the wildcard list
#[delete("/dns/wildlist/<domain>")]
pub fn delete_wildlist(config: State<Config>, domain: String) -> util::Reply {
    remove_list(List::Wildlist, &domain, &config)?;
    reload_gravity(List::Wildlist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}
