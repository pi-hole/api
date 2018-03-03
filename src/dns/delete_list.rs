/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for removing domains from lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use dns::common::reload_gravity;
use dns::list::{List, remove_list};
use util;

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(domain: String) -> util::Reply {
    remove_list(List::Whitelist, &domain)?;
    reload_gravity(List::Whitelist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(domain: String) -> util::Reply {
    remove_list(List::Blacklist, &domain)?;
    reload_gravity(List::Blacklist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the wildcard list
#[delete("/dns/wildlist/<domain>")]
pub fn delete_wildlist(domain: String) -> util::Reply {
    remove_list(List::Wildlist, &domain)?;
    reload_gravity(List::Wildlist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}
