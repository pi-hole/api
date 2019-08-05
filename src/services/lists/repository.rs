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
    services::lists::List,
    util::{Error, ErrorKind}
};
use diesel::{delete, dsl::exists, insert_into, prelude::*, select};
use failure::ResultExt;
use rocket::{
    request::{self, FromRequest},
    Outcome, Request
};
use std::marker::PhantomData;

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

/// The implementation of `ListRepository`
pub struct ListRepositoryImpl<'r> {
    db: GravityDatabase,
    phantom: PhantomData<&'r ()>
}

impl<'r> ListRepositoryImpl<'r> {
    fn new(db: GravityDatabase) -> Self {
        ListRepositoryImpl {
            db,
            phantom: PhantomData
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for ListRepositoryImpl<'r> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let db = request.guard::<GravityDatabase>()?;
        Outcome::Success(ListRepositoryImpl::new(db))
    }
}

impl<'r> ListRepository for ListRepositoryImpl<'r> {
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
                select(exists(
                    whitelist
                        .filter(enabled.eq(true))
                        .filter(domain.eq(input_domain))
                ))
                .get_result(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;
                select(exists(
                    blacklist
                        .filter(enabled.eq(true))
                        .filter(domain.eq(input_domain))
                ))
                .get_result(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;
                select(exists(
                    regex
                        .filter(enabled.eq(true))
                        .filter(domain.eq(input_domain))
                ))
                .get_result(db)
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
                    .values(&(domain.eq(input_domain), enabled.eq(true)))
                    .execute(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;
                insert_into(blacklist)
                    .values(&(domain.eq(input_domain), enabled.eq(true)))
                    .execute(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;
                insert_into(regex)
                    .values(&(domain.eq(input_domain), enabled.eq(true)))
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
                delete(
                    whitelist
                        .filter(enabled.eq(true))
                        .filter(domain.eq(input_domain))
                )
                .execute(db)
            }
            List::Black => {
                use crate::databases::gravity::blacklist::dsl::*;
                delete(
                    blacklist
                        .filter(enabled.eq(true))
                        .filter(domain.eq(input_domain))
                )
                .execute(db)
            }
            List::Regex => {
                use crate::databases::gravity::regex::dsl::*;
                delete(
                    regex
                        .filter(enabled.eq(true))
                        .filter(domain.eq(input_domain))
                )
                .execute(db)
            }
        }
        .context(ErrorKind::GravityDatabase)?;

        Ok(())
    }
}

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
impl Default for ListRepositoryMock {
    fn default() -> Self {
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

#[cfg(test)]
mod tests {
    use super::{ListRepository, ListRepositoryImpl};
    use crate::{databases::gravity::connect_to_gravity_test_db, services::lists::List};

    /// Assert that the list of domains retrieved from the database equals the
    /// expected list
    fn get_test(list: List, expected_domains: Vec<String>) {
        let db = connect_to_gravity_test_db();
        let repo = ListRepositoryImpl::new(db);

        let actual_domains = repo.get(list).unwrap();

        assert_eq!(actual_domains, expected_domains);
    }

    /// Assert that the list contains the given domain
    fn contains_test(list: List, domain: &str) {
        let db = connect_to_gravity_test_db();
        let repo = ListRepositoryImpl::new(db);

        assert!(repo.contains(list, domain).unwrap())
    }

    /// Assert that adding a domain not already on the list works
    fn add_test(list: List, domain: &str) {
        let db = connect_to_gravity_test_db();
        let repo = ListRepositoryImpl::new(db);

        // Make sure it doesn't exist already
        let initial_domains = repo.get(list).unwrap();
        assert!(!initial_domains.contains(&domain.to_owned()));

        repo.add(list, domain).unwrap();

        // Make sure it was added
        let domains = repo.get(list).unwrap();
        assert!(domains.contains(&domain.to_owned()));
    }

    /// Assert that deleting a domain from the list works
    fn delete_test(list: List, domain: &str) {
        let db = connect_to_gravity_test_db();
        let repo = ListRepositoryImpl::new(db);

        // Make sure the domain is on the list
        let domains = repo.get(list).unwrap();
        assert!(domains.contains(&domain.to_owned()));

        repo.remove(list, domain).unwrap();

        // Make sure it was removed
        let domains = repo.get(list).unwrap();
        assert!(!domains.contains(&domain.to_owned()));
    }

    /// Getting the lists should return the expected domains
    #[test]
    fn get() {
        get_test(List::White, vec!["test.com".to_owned()]);
        get_test(List::Black, vec!["example.com".to_owned()]);
        get_test(List::Regex, vec!["(^|\\.)example\\.com$".to_owned()]);
    }

    /// Assert that checking for an existing domain works
    #[test]
    fn contains_existing() {
        contains_test(List::White, "test.com");
        contains_test(List::Black, "example.com");
        contains_test(List::Regex, "(^|\\.)example\\.com$");
    }

    /// Adding new domains to the lists should add the domains
    #[test]
    fn add_new() {
        add_test(List::White, "whitelist.com");
        add_test(List::Black, "blacklist.com");
        add_test(List::Regex, "regex.com");
    }

    /// Deleting existing domains from the lists should work
    #[test]
    fn delete_existing() {
        delete_test(List::White, "test.com");
        delete_test(List::Black, "example.com");
        delete_test(List::Regex, "(^|\\.)example\\.com$");
    }
}
