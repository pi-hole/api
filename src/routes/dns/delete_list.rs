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
    env::Env,
    ftl::FtlConnectionType,
    lists::{List, ListRepositoryGuard},
    routes::auth::User,
    util::{reply_success, Reply}
};
use rocket::State;

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(
    _auth: User,
    env: State<Env>,
    repo: ListRepositoryGuard,
    ftl: State<FtlConnectionType>,
    domain: String
) -> Reply {
    List::White.remove(&domain, &env, &*repo, &ftl)?;
    reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(
    _auth: User,
    env: State<Env>,
    repo: ListRepositoryGuard,
    ftl: State<FtlConnectionType>,
    domain: String
) -> Reply {
    List::Black.remove(&domain, &env, &*repo, &ftl)?;
    reply_success()
}

/// Delete a domain from the regex list
#[delete("/dns/regexlist/<domain>")]
pub fn delete_regexlist(
    _auth: User,
    env: State<Env>,
    repo: ListRepositoryGuard,
    ftl: State<FtlConnectionType>,
    domain: String
) -> Reply {
    List::Regex.remove(&domain, &env, &*repo, &ftl)?;
    reply_success()
}
