// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Foreign key enabled SQLite connection
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use diesel::{r2d2, Connection};
use rocket_contrib::databases::{DatabaseConfig, Poolable};
use std::ops::{Deref, DerefMut};

/// A wrapper around `SqliteConnection` for use by `SqliteFKConnectionManager`
pub struct SqliteFKConnection(diesel::SqliteConnection);

impl Deref for SqliteFKConnection {
    type Target = diesel::SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SqliteFKConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// This implementation is mostly copied from the implementation for
// `SqliteConnection`, but uses `SqliteFKConnectionManager`
impl Poolable for SqliteFKConnection {
    type Manager = SqliteFKConnectionManager;
    type Error = rocket_contrib::databases::r2d2::Error;

    fn pool(config: DatabaseConfig) -> Result<r2d2::Pool<Self::Manager>, Self::Error> {
        let manager = SqliteFKConnectionManager::new(config.url);
        r2d2::Pool::builder()
            .max_size(config.pool_size)
            .build(manager)
    }
}

/// A SQLite connection manager which automatically turns on foreign key support
pub struct SqliteFKConnectionManager(pub diesel::r2d2::ConnectionManager<diesel::SqliteConnection>);

impl SqliteFKConnectionManager {
    pub fn new(database_url: &str) -> Self {
        SqliteFKConnectionManager(diesel::r2d2::ConnectionManager::new(database_url))
    }
}

impl r2d2::ManageConnection for SqliteFKConnectionManager {
    type Connection = SqliteFKConnection;
    type Error = r2d2::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = self.0.connect()?;

        // Turn on foreign key support
        conn.execute("PRAGMA FOREIGN_KEYS=ON")
            .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(SqliteFKConnection(conn))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        self.0.is_valid(conn)
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        self.0.has_broken(conn)
    }
}
