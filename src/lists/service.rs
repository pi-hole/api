// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// List Service (Whitelist, Blacklist, Regexlist)
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::FtlConnectionType,
    lists::ListRepository,
    settings::ValueType,
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use std::process::{Command, Stdio};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum List {
    White,
    Black,
    Regex
}

impl List {
    /// Add a domain to the list and update FTL and other lists accordingly.
    /// Example: when adding to the whitelist, remove from the blacklist.
    pub fn add(
        &self,
        domain: &str,
        env: &Env,
        repo: &dyn ListRepository,
        ftl: &FtlConnectionType
    ) -> Result<(), Error> {
        match self {
            List::White => {
                // We need to add it to the whitelist and remove it from the
                // blacklist
                List::White.add_raw(domain, repo)?;
                List::Black.try_remove_raw(domain, repo)?;

                // Since we haven't hit an error yet, reload gravity
                reload_gravity(List::White, env)
            }
            List::Black => {
                // We need to add it to the blacklist and remove it from the
                // whitelist
                List::Black.add_raw(domain, repo)?;
                List::White.try_remove_raw(domain, repo)?;

                // Since we haven't hit an error yet, reload gravity
                reload_gravity(List::Black, env)
            }
            List::Regex => {
                // We only need to add it to the regex list
                List::Regex.add_raw(domain, repo)?;

                // Since we haven't hit an error yet, tell FTL to recompile
                // regex
                ftl.connect("recompile-regex")?.expect_eom()
            }
        }
    }

    /// Remove a domain from the list and update FTL
    pub fn remove(
        &self,
        domain: &str,
        env: &Env,
        repo: &dyn ListRepository,
        ftl: &FtlConnectionType
    ) -> Result<(), Error> {
        match self {
            List::White => {
                List::White.remove_raw(domain, repo)?;
                reload_gravity(List::White, env)
            }
            List::Black => {
                List::Black.remove_raw(domain, repo)?;
                reload_gravity(List::Black, env)
            }
            List::Regex => {
                List::Regex.remove_raw(domain, repo)?;
                ftl.connect("recompile-regex")?.expect_eom()
            }
        }
    }

    /// Add a domain to the list
    fn add_raw(&self, domain: &str, repo: &dyn ListRepository) -> Result<(), Error> {
        // Check if it's a valid domain before doing anything
        if !self.accepts(domain) {
            return Err(Error::from(ErrorKind::InvalidDomain));
        }

        // Check if the domain is already in the list
        if repo.contains(*self, domain)? {
            return Err(Error::from(ErrorKind::AlreadyExists));
        }

        repo.add(*self, domain)
    }

    /// Try to remove a domain from the list, but it is not an error if the
    /// domain does not exist
    pub fn try_remove_raw(&self, domain: &str, repo: &dyn ListRepository) -> Result<(), Error> {
        match self.remove_raw(domain, repo) {
            // Pass through successful results
            Ok(_) => Ok(()),
            Err(e) => {
                // Ignore NotFound errors
                if e.kind() == ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Remove a domain from the list
    pub fn remove_raw(&self, domain: &str, repo: &dyn ListRepository) -> Result<(), Error> {
        // Check if it's a valid domain before doing anything
        if !self.accepts(domain) {
            return Err(Error::from(ErrorKind::InvalidDomain));
        }

        // Check if the domain is not in the list
        if !repo.contains(*self, domain)? {
            return Err(Error::from(ErrorKind::NotFound));
        }

        repo.remove(*self, domain)
    }

    /// Check if the list accepts the domain as valid
    fn accepts(&self, domain: &str) -> bool {
        match self {
            List::Regex => ValueType::Regex.is_valid(domain),
            // Allow hostnames to be white/blacklist-ed
            _ => ValueType::Hostname.is_valid(domain)
        }
    }
}

/// Reload Gravity to activate changes in lists
pub fn reload_gravity(list: List, env: &Env) -> Result<(), Error> {
    // Don't actually reload Gravity during testing
    if env.is_test() {
        return Ok(());
    }

    let status = Command::new("sudo")
        .arg("pihole")
        .arg("-g")
        .arg("--skip-download")
        // Based on what list we modified, only reload what is necessary
        .arg(match list {
            List::White => "--whitelist-only",
            List::Black => "--blacklist-only",
            _ => return Err(Error::from(ErrorKind::Unknown))
        })
        // Ignore stdin, stdout, and stderr
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Get the returned status code
        .status()
        .context(ErrorKind::GravityError)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::from(ErrorKind::GravityError))
    }
}

#[cfg(test)]
mod test {
    use super::List;
    use crate::{
        ftl::FtlConnectionType,
        lists::ListRepositoryMock,
        testing::{write_eom, TestEnvBuilder}
    };
    use mock_it::verify;
    use std::collections::HashMap;

    fn get_ftl() -> FtlConnectionType {
        let mut data = Vec::new();
        let mut command_map = HashMap::new();

        write_eom(&mut data);
        command_map.insert("recompile-regex".to_owned(), data);

        FtlConnectionType::Test(command_map)
    }

    //    /// The whitelist is retrieved correctly
    //    #[test]
    //    fn get_whitelist() {
    //        let scenario = Scenario::new();
    //        let repo = scenario.create_mock_for::<ListRepository>();
    //
    //        scenario.expect(
    //            repo.get_call(List::White)
    //                .and_return(Ok(vec!["whitelist.com".to_owned()]))
    //        );
    //
    //        assert_eq!(
    //            List::White.get(&repo).unwrap(),
    //            vec!["whitelist.com".to_owned()]
    //        );
    //    }

    //    /// The blacklist is retrieved correctly
    //    #[test]
    //    fn get_blacklist() {
    //        let db = create_test_db();
    //
    //        assert_eq!(List::Black.get(&db).unwrap(), vec!["blacklist.com"]);
    //    }
    //
    //    /// The regexlist is retrieved when it exists
    //    #[test]
    //    fn get_regexlist() {
    //        let db = create_test_db();
    //
    //        assert_eq!(List::Regex.get(&db).unwrap(), vec!["regex.com"]);
    //    }

    /// Adding a domain to the whitelist works when the domain does not exist
    /// in either the whitelist or blacklist
    #[test]
    fn add_whitelist() {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let repo = ListRepositoryMock::new();

        repo.contains
            .given((List::White, "example.com".to_owned()))
            .will_return(Ok(false));
        repo.add
            .given((List::White, "example.com".to_owned()))
            .will_return(Ok(()));
        repo.contains
            .given((List::Black, "example.com".to_owned()))
            .will_return(Ok(false));

        List::White.add("example.com", &env, &repo, &ftl).unwrap();

        verify(
            repo.add
                .was_called_with((List::White, "example.com".to_owned()))
        );
    }

    /// Adding a domain to the blacklist works when the domain does not exist
    /// in either the whitelist or blacklist
    #[test]
    fn add_blacklist() {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let repo = ListRepositoryMock::new();

        repo.contains
            .given((List::Black, "example.com".to_owned()))
            .will_return(Ok(false));
        repo.add
            .given((List::Black, "example.com".to_owned()))
            .will_return(Ok(()));
        repo.contains
            .given((List::White, "example.com".to_owned()))
            .will_return(Ok(false));

        List::Black.add("example.com", &env, &repo, &ftl).unwrap();

        verify(
            repo.add
                .was_called_with((List::Black, "example.com".to_owned()))
        );
    }

    /// Adding a domain to the regex list works when the domain does not already
    /// exist in the regex list
    #[test]
    fn add_regexlist() {
        let env = TestEnvBuilder::new().build();
        let ftl = get_ftl();
        let repo = ListRepositoryMock::new();

        repo.contains
            .given((List::Regex, "example.com".to_owned()))
            .will_return(Ok(false));
        repo.add
            .given((List::Regex, "example.com".to_owned()))
            .will_return(Ok(()));

        List::Regex.add("example.com", &env, &repo, &ftl).unwrap();

        verify(
            repo.add
                .was_called_with((List::Regex, "example.com".to_owned()))
        );
    }

    //    #[test]
    //    fn delete_whitelist() {
    //        let env = TestEnvBuilder::new().build();
    //        let db = connect_to_gravity_test_db();
    //        let ftl = get_ftl();
    //
    //        List::White.remove("whitelist.com", &env, &db, &ftl).unwrap();
    //
    //        assert_eq!(
    //            List::White.get(&db).unwrap().len(),
    //            0
    //        );
    //    }
    //
    //    #[test]
    //    fn delete_blacklist() {
    //        let env = TestEnvBuilder::new().build();
    //        let db = connect_to_gravity_test_db();
    //        let ftl = get_ftl();
    //
    //        List::Black.remove("blacklist.com", &env, &db, &ftl).unwrap();
    //
    //        assert_eq!(
    //            List::Black.get(&db).unwrap().len(),
    //            0
    //        );
    //    }
    //
    //    #[test]
    //    fn delete_regexlist() {
    //        let env = TestEnvBuilder::new().build();
    //        let db = connect_to_gravity_test_db();
    //        let ftl = get_ftl();
    //
    //        List::Regex.remove("regex.com", &env, &db, &ftl).unwrap();
    //
    //        assert_eq!(
    //            List::Regex.get(&db).unwrap().len(),
    //            0
    //        );
    //    }
}
