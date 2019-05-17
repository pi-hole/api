// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Reading Domain Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    lists::{List, ListRepositoryGuard},
    util::{reply_result, Reply}
};

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(repo: ListRepositoryGuard) -> Reply {
    reply_result(repo.get(List::White))
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(repo: ListRepositoryGuard) -> Reply {
    reply_result(repo.get(List::Black))
}

/// Get the Regex list domains
#[get("/dns/regexlist")]
pub fn get_regexlist(repo: ListRepositoryGuard) -> Reply {
    reply_result(repo.get(List::Regex))
}
