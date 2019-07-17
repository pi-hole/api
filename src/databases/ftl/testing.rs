// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Test Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::databases::{
    common::{create_memory_db, start_test_transaction},
    foreign_key_connection::SqliteFKConnectionManager,
    ftl::FtlDatabase
};
use diesel::{r2d2::Pool, SqliteConnection};

pub const TEST_FTL_DATABASE_SCHEMA: &str = include_str!("../../../test/FTL.sql");

lazy_static! {
    /// A connection pool for tests which need a database connection
    static ref CONNECTION_POOL: Pool<SqliteFKConnectionManager> = {
        create_memory_db(TEST_FTL_DATABASE_SCHEMA, 8)
    };
}

/// Connect to the testing database
pub fn connect_to_ftl_test_db() -> FtlDatabase {
    let db = FtlDatabase(CONNECTION_POOL.get().unwrap());
    start_test_transaction(&db as &SqliteConnection);

    db
}
