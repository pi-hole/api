// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Foreign Key Enabled SQLite Connection
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use diesel::{
    connection::SimpleConnection,
    r2d2::{self, ConnectionManager, CustomizeConnection, Pool},
    Connection, SqliteConnection
};
use rocket::config::Value;
use rocket_contrib::databases::{DatabaseConfig, Poolable};
use std::ops::{Deref, DerefMut};

/// A wrapper around `SqliteConnection` for use by `SqliteFKConnectionManager`
pub struct SqliteFKConnection(SqliteConnection);

// Implement the dereference traits so it can be used in place of a normal
// SqliteConnection
impl Deref for SqliteFKConnection {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SqliteFKConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Poolable for SqliteFKConnection {
    type Manager = SqliteFKConnectionManager;
    type Error = rocket_contrib::databases::r2d2::Error;

    fn pool(config: DatabaseConfig) -> Result<Pool<Self::Manager>, Self::Error> {
        let manager = SqliteFKConnectionManager(ConnectionManager::new(config.url));
        let mut builder = Pool::builder().max_size(config.pool_size);

        // When testing, run the schema SQL to build the database
        if cfg!(test) {
            if let Some(Value::String(schema)) = config.extras.get("test_schema").cloned() {
                builder = builder.connection_customizer(Box::new(DatabaseSchemaApplier { schema }));
            }
        }

        builder.build(manager)
    }
}

/// A SQLite connection manager which automatically turns on foreign key support
pub struct SqliteFKConnectionManager(pub ConnectionManager<SqliteConnection>);

impl r2d2::ManageConnection for SqliteFKConnectionManager {
    type Connection = SqliteFKConnection;
    type Error = r2d2::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = self.0.connect()?;

        // Turn on foreign key support
        conn.execute("PRAGMA FOREIGN_KEYS=ON")
            .map_err(r2d2::Error::QueryError)?;

        Ok(SqliteFKConnection(conn))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        self.0.is_valid(conn)
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        self.0.has_broken(conn)
    }
}

/// Applies a schema to the database after connecting
#[derive(Debug)]
struct DatabaseSchemaApplier {
    schema: String
}

impl CustomizeConnection<SqliteFKConnection, r2d2::Error> for DatabaseSchemaApplier {
    fn on_acquire(&self, conn: &mut SqliteFKConnection) -> Result<(), r2d2::Error> {
        // Apply the schema in a transaction
        conn.transaction(|| conn.batch_execute(&self.schema))
            .map_err(r2d2::Error::QueryError)
    }
}
