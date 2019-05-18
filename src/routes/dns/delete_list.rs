// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Removing Domains From Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    lists::{List, ListServiceGuard},
    routes::auth::User,
    util::{reply_success, Reply}
};

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(_auth: User, list_service: ListServiceGuard, domain: String) -> Reply {
    list_service.remove(List::White, &domain)?;
    reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(_auth: User, list_service: ListServiceGuard, domain: String) -> Reply {
    list_service.remove(List::Black, &domain)?;
    reply_success()
}

/// Delete a domain from the regex list
#[delete("/dns/regexlist/<domain>")]
pub fn delete_regexlist(_auth: User, list_service: ListServiceGuard, domain: String) -> Reply {
    list_service.remove(List::Regex, &domain)?;
    reply_success()
}
