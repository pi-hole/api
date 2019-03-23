// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Databases
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    settings::{ConfigEntry, FtlConfEntry},
    util::Error
};
use rocket::config::Value;
use std::collections::HashMap;

#[cfg(test)]
use crate::databases::ftl::TEST_FTL_DATABASE_PATH;

pub mod ftl;

/// Load the database URLs from the API config into the Rocket config format
pub fn load_databases(env: &Env) -> Result<HashMap<&str, HashMap<&str, Value>>, Error> {
    let mut databases = HashMap::new();
    let mut ftl_database = HashMap::new();

    ftl_database.insert("url", Value::from(FtlConfEntry::DbFile.read(env)?));
    databases.insert("ftl_database", ftl_database);

    Ok(databases)
}

/// Load test database URLs into the Rocket config format
#[cfg(test)]
pub fn load_test_databases() -> HashMap<&'static str, HashMap<&'static str, Value>> {
    let mut databases = HashMap::new();
    let mut ftl_database = HashMap::new();

    ftl_database.insert("url", Value::from(TEST_FTL_DATABASE_PATH));
    databases.insert("ftl_database", ftl_database);

    databases
}
