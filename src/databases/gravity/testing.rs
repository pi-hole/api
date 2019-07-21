// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Gravity Database Test Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::databases::{common::create_memory_db, gravity::GravityDatabase};

pub const TEST_GRAVITY_DATABASE_SCHEMA: &str = include_str!("../../../test/gravity.sql");

/// Connect to the testing database. This creates a new in-memory database so
/// that it is isolated from other tests.
pub fn connect_to_gravity_test_db() -> GravityDatabase {
    let pool = create_memory_db(TEST_GRAVITY_DATABASE_SCHEMA, 1);

    GravityDatabase(pool.get().unwrap())
}
