// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Gravity Database Test Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::databases::{
    common::start_test_transaction, foreign_key_connection::SqliteFKConnectionManager,
    gravity::GravityDatabase
};
use diesel::{r2d2::Pool, SqliteConnection};

pub const TEST_GRAVITY_DATABASE_PATH: &str = "test/gravity.db";

lazy_static! {
    /// A connection pool for tests which need a database connection
    static ref CONNECTION_POOL: Pool<SqliteFKConnectionManager> = {
        let manager = SqliteFKConnectionManager::new(TEST_GRAVITY_DATABASE_PATH);
        diesel::r2d2::Pool::builder().max_size(1).build(manager).unwrap()
    };
}

/// Connect to the testing database
pub fn connect_to_gravity_test_db() -> GravityDatabase {
    let db = GravityDatabase(CONNECTION_POOL.get().unwrap());
    start_test_transaction(&db as &SqliteConnection);

    db
}
