// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common database functions
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
use crate::databases::{
    custom_connection::{CustomSqliteConnection, CustomSqliteConnectionManager},
    ftl::TEST_FTL_DATABASE_SCHEMA,
    gravity::TEST_GRAVITY_DATABASE_SCHEMA
};
#[cfg(test)]
use diesel::{
    connection::{Connection, TransactionManager},
    r2d2::Pool,
    SqliteConnection
};
#[cfg(test)]
use rocket_contrib::databases::{DatabaseConfig, Poolable};
#[cfg(test)]
use std::collections::BTreeMap;

/// Load the database URLs from the API config into the Rocket config format
pub fn load_databases(env: &Env) -> Result<HashMap<&str, HashMap<&str, Value>>, Error> {
    let mut databases = HashMap::new();
    let mut ftl_database = HashMap::new();
    let mut gravity_database = HashMap::new();

    ftl_database.insert("url", Value::from(FtlConfEntry::DbFile.read(env)?));
    gravity_database.insert("url", Value::from(FtlConfEntry::GravityDb.read(env)?));

    databases.insert("ftl_database", ftl_database);
    databases.insert("gravity_database", gravity_database);

    Ok(databases)
}

/// Load test database URLs into the Rocket config format
#[cfg(test)]
pub fn load_test_databases() -> HashMap<&'static str, HashMap<&'static str, Value>> {
    let mut databases = HashMap::new();
    let mut ftl_database = HashMap::new();
    let mut gravity_database = HashMap::new();

    ftl_database.insert("url", Value::from(":memory:"));
    ftl_database.insert("pool_size", Value::from(8));
    ftl_database.insert("test_schema", Value::from(TEST_FTL_DATABASE_SCHEMA));

    gravity_database.insert("url", Value::from(":memory:"));
    gravity_database.insert("pool_size", Value::from(1));
    gravity_database.insert("test_schema", Value::from(TEST_GRAVITY_DATABASE_SCHEMA));

    databases.insert("ftl_database", ftl_database);
    databases.insert("gravity_database", gravity_database);

    databases
}

/// Start a test transaction so the database does not get modified. If a
/// transaction is already running, it is rolled back.
#[cfg(test)]
pub fn start_test_transaction(db: &SqliteConnection) {
    let transaction_manager: &TransactionManager<SqliteConnection> = db.transaction_manager();
    let depth = transaction_manager.get_transaction_depth();

    if depth > 0 {
        transaction_manager.rollback_transaction(db).unwrap();
    }

    db.begin_test_transaction().unwrap();
}

/// Create an in-memory SQLite database with the given schema (SQL commands)
#[cfg(test)]
pub fn create_memory_db(schema: &str, pool_size: u32) -> Pool<CustomSqliteConnectionManager> {
    let mut extras = BTreeMap::new();
    extras.insert("test_schema".to_owned(), Value::from(schema));

    let config = DatabaseConfig {
        url: ":memory:",
        pool_size,
        extras
    };

    CustomSqliteConnection::pool(config).unwrap()
}
