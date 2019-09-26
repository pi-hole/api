// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Custom SQLite Connection
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use diesel::{
    connection::SimpleConnection,
    r2d2::{self, ConnectionManager, CustomizeConnection, Pool},
    Connection, ConnectionError, SqliteConnection
};
use rocket::config::Value;
use rocket_contrib::databases::{DatabaseConfig, Poolable};
use std::{
    ops::{Deref, DerefMut},
    path::Path
};

/// A wrapper around `SqliteConnection` for use by
/// `CustomSqliteConnectionManager`
pub struct CustomSqliteConnection(SqliteConnection);

// Implement the dereference traits so it can be used in place of a normal
// SqliteConnection
impl Deref for CustomSqliteConnection {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CustomSqliteConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Poolable for CustomSqliteConnection {
    type Manager = CustomSqliteConnectionManager;
    type Error = rocket_contrib::databases::r2d2::Error;

    fn pool(config: DatabaseConfig) -> Result<Pool<Self::Manager>, Self::Error> {
        let manager = CustomSqliteConnectionManager {
            manager: ConnectionManager::new(config.url),
            database_url: config.url.to_owned()
        };
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

/// A custom SQLite connection manager which automatically adds a busy timeout
/// and fails if the database does not exist
pub struct CustomSqliteConnectionManager {
    pub manager: ConnectionManager<SqliteConnection>,
    database_url: String
}

impl r2d2::ManageConnection for CustomSqliteConnectionManager {
    type Connection = CustomSqliteConnection;
    type Error = r2d2::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        // Don't connect to missing databases. If we did connect, it would make
        // a zero-sized DB without a schema. This would mess up other services.
        if self.database_url != ":memory:" && !Path::new(&self.database_url).exists() {
            return Err(r2d2::Error::ConnectionError(
                ConnectionError::BadConnection(format!("{} does not exist", self.database_url))
            ));
        }

        let conn = self.manager.connect()?;

        // Add a busy timeout of one second
        conn.execute("PRAGMA busy_timeout = 1000")
            .map_err(r2d2::Error::QueryError)?;

        Ok(CustomSqliteConnection(conn))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        self.manager.is_valid(conn)
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        self.manager.has_broken(conn)
    }
}

/// Applies a schema to the database after connecting
#[derive(Debug)]
struct DatabaseSchemaApplier {
    schema: String
}

impl CustomizeConnection<CustomSqliteConnection, r2d2::Error> for DatabaseSchemaApplier {
    fn on_acquire(&self, conn: &mut CustomSqliteConnection) -> Result<(), r2d2::Error> {
        // Apply the schema in a transaction
        conn.transaction(|| conn.batch_execute(&self.schema))
            .map_err(r2d2::Error::QueryError)
    }
}
