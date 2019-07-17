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
use crate::databases::{foreign_key_connection::SqliteFKConnectionManager, start_test_transaction};
#[cfg(test)]
use diesel::{r2d2::Pool, SqliteConnection};

mod model;
mod schema;

pub use self::{model::*, schema::*};

#[cfg(test)]
pub const TEST_FTL_DATABASE_PATH: &str = "test/FTL.db";

#[cfg(test)]
lazy_static! {
    /// A connection pool for tests which need a database connection
    static ref CONNECTION_POOL: Pool<SqliteFKConnectionManager> = {
        let manager = SqliteFKConnectionManager::new(TEST_FTL_DATABASE_PATH);
        diesel::r2d2::Pool::builder().build(manager).unwrap()
    };
}

/// Connect to the testing database
#[cfg(test)]
pub fn connect_to_ftl_test_db() -> FtlDatabase {
    let db = FtlDatabase(CONNECTION_POOL.get().unwrap());
    start_test_transaction(&db as &SqliteConnection);

    db
}
