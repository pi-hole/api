// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Support
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

#[cfg(test)]
use diesel::{sqlite::SqliteConnection, Connection};

mod model;
mod schema;

pub use self::{model::*, schema::*};

#[cfg(test)]
pub const TEST_FTL_DATABASE_PATH: &str = "test/FTL.db";

/// Connect to the testing database
#[cfg(test)]
pub fn connect_to_ftl_test_db() -> SqliteConnection {
    SqliteConnection::establish(TEST_FTL_DATABASE_PATH).unwrap()
}
