// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// List Database Repository
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::gravity::GravityDatabase,
    lists::List,
    util::{Error, ErrorKind}
};
use diesel::{delete, dsl::exists, insert_into, prelude::*, select};
use failure::ResultExt;
use rocket::{
    request::{self, FromRequest},
    Outcome, Request
};
use std::ops::Deref;

#[cfg(test)]
use mock_it::Mock;

/// Describes interactions with the list data store
pub trait ListRepository {
    /// Get all of the domains in the list
    fn get(&self, list: List) -> Result<Vec<String>, Error>;

    /// Check if the list contains the domain
    fn contains(&self, list: List, domain: &str) -> Result<bool, Error>;

    /// Add the domain to the list
    fn add(&self, list: List, domain: &str) -> Result<(), Error>;

    /// Remove the domain from the list
    fn remove(&self, list: List, domain: &str) -> Result<(), Error>;
}

service!(
    ListRepositoryGuard,
    ListRepository,
    ListRepositoryImpl,
    ListRepositoryMock
);

// TODO: add proc macro to mocking library to generate the mock
#[cfg(test)]
#[derive(Clone)]
pub struct ListRepositoryMock {
    pub get: Mock<List, Result<Vec<String>, Error>>,
    pub contains: Mock<(List, String), Result<bool, Error>>,
    pub add: Mock<(List, String), Result<(), Error>>,
    pub remove: Mock<(List, String), Result<(), Error>>
}

#[cfg(test)]
impl ListRepositoryMock {
    pub fn new() -> Self {
        ListRepositoryMock {
            get: Mock::new(Ok(Vec::new())),
            contains: Mock::new(Ok(false)),
            add: Mock::new(Ok(())),
            remove: Mock::new(Ok(()))
        }
    }
}

#[cfg(test)]
impl ListRepository for ListRepositoryMock {
    fn get(&self, list: List) -> Result<Vec<String>, Error> {
        self.get.called(list)
    }

    fn contains(&self, list: List, domain: &str) -> Result<bool, Error> {
        self.contains.called((list, domain.to_owned()))
    }

    fn add(&self, list: List, domain: &str) -> Result<(), Error> {
        self.add.called((list, domain.to_owned()))
    }

    fn remove(&self, list: List, domain: &str) -> Result<(), Error> {
        self.remove.called((list, domain.to_owned()))
    }
}

pub struct ListRepositoryImpl {
    db: GravityDatabase
}

impl<'a, 'r> FromRequest<'a, 'r> for ListRepositoryImpl {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let db = request.guard::<GravityDatabase>()?;
        Outcome::Success(ListRepositoryImpl { db })
    }
}

impl ListRepository for ListRepositoryImpl {
    fn get(&self, list: List) -> Result<Vec<String>, Error> {
        let db = &self.db as &SqliteConnection;

        match list {
            List::White => {
                use crate::databases::gravity::whitelist::dsl::*;

                whitelist.select(domain).filter(enabled.eq(true)).load(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;

                blacklist.select(domain).filter(enabled.eq(true)).load(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;

                regex.select(domain).filter(enabled.eq(true)).load(db)
            }
        }
        .context(ErrorKind::GravityDatabase)
        .map_err(Error::from)
    }

    fn contains(&self, list: List, input_domain: &str) -> Result<bool, Error> {
        let db = &self.db as &SqliteConnection;

        match list {
            List::White => {
                use crate::databases::gravity::whitelist::dsl::*;

                select(exists(whitelist.filter(domain.eq(input_domain)))).get_result(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;

                select(exists(blacklist.filter(domain.eq(input_domain)))).get_result(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;

                select(exists(regex.filter(domain.eq(input_domain)))).get_result(db)
            }
        }
        .context(ErrorKind::GravityDatabase)
        .map_err(Error::from)
    }

    fn add(&self, list: List, input_domain: &str) -> Result<(), Error> {
        let db = &self.db as &SqliteConnection;

        match list {
            List::White => {
                use crate::databases::gravity::whitelist::dsl::*;

                insert_into(whitelist)
                    .values(&domain.eq(input_domain))
                    .execute(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;

                insert_into(blacklist)
                    .values(&domain.eq(input_domain))
                    .execute(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;

                insert_into(regex)
                    .values(&domain.eq(input_domain))
                    .execute(db)
            }
        }
        .context(ErrorKind::GravityDatabase)?;

        Ok(())
    }

    fn remove(&self, list: List, input_domain: &str) -> Result<(), Error> {
        let db = &self.db as &SqliteConnection;

        match list {
            List::White => {
                use crate::databases::gravity::whitelist::dsl::*;

                delete(whitelist.filter(domain.eq(input_domain))).execute(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;

                delete(blacklist.filter(domain.eq(input_domain))).execute(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;

                delete(regex.filter(domain.eq(input_domain))).execute(db)
            }
        }
        .context(ErrorKind::GravityDatabase)?;

        Ok(())
    }
}
