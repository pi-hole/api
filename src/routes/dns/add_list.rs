// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Adding Domains To Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::FtlConnectionType,
    lists::{List, ListRepositoryGuard},
    routes::auth::User,
    util::{reply_success, Reply}
};
use rocket::State;
use rocket_contrib::json::Json;

/// Represents an API input containing a domain
#[derive(Deserialize)]
pub struct DomainInput {
    domain: String
}

/// Add a domain to the whitelist
#[post("/dns/whitelist", data = "<domain_input>")]
pub fn add_whitelist(
    _auth: User,
    env: State<Env>,
    repo: ListRepositoryGuard,
    ftl: State<FtlConnectionType>,
    domain_input: Json<DomainInput>
) -> Reply {
    List::White.add(&domain_input.0.domain, &env, &*repo, &ftl)?;
    reply_success()
}

/// Add a domain to the blacklist
#[post("/dns/blacklist", data = "<domain_input>")]
pub fn add_blacklist(
    _auth: User,
    env: State<Env>,
    repo: ListRepositoryGuard,
    ftl: State<FtlConnectionType>,
    domain_input: Json<DomainInput>
) -> Reply {
    List::Black.add(&domain_input.0.domain, &env, &*repo, &ftl)?;
    reply_success()
}

/// Add a domain to the regex list
#[post("/dns/regexlist", data = "<domain_input>")]
pub fn add_regexlist(
    _auth: User,
    env: State<Env>,
    repo: ListRepositoryGuard,
    ftl: State<FtlConnectionType>,
    domain_input: Json<DomainInput>
) -> Reply {
    List::Regex.add(&domain_input.0.domain, &env, &*repo, &ftl)?;
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::{
        lists::{List, ListRepositoryMock},
        testing::{write_eom, TestBuilder}
    };
    use mock_it::verify;
    use rocket::http::Method;

    #[test]
    fn add_whitelist() {
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

        TestBuilder::new()
            .endpoint("/admin/api/dns/whitelist")
            .method(Method::Post)
            .mock_service(repo.clone())
            .body(json!({ "domain": "example.com" }))
            .expect_json(json!({ "status": "success" }))
            .test();

        assert!(verify(
            repo.add
                .was_called_with((List::White, "example.com".to_owned()))
        ));
    }

    #[test]
    fn add_blacklist() {
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

        TestBuilder::new()
            .endpoint("/admin/api/dns/blacklist")
            .method(Method::Post)
            .mock_service(repo.clone())
            .body(json!({ "domain": "example.com" }))
            .expect_json(json!({ "status": "success" }))
            .test();

        assert!(verify(
            repo.add
                .was_called_with((List::Black, "example.com".to_owned()))
        ));
    }

    #[test]
    fn test_add_regexlist() {
        let mut data = Vec::new();
        write_eom(&mut data);

        let repo = ListRepositoryMock::new();

        repo.contains
            .given((List::Regex, "^.*example.com$".to_owned()))
            .will_return(Ok(false));
        repo.add
            .given((List::Regex, "^.*example.com$".to_owned()))
            .will_return(Ok(()));

        TestBuilder::new()
            .endpoint("/admin/api/dns/regexlist")
            .method(Method::Post)
            .ftl("recompile-regex", data)
            .mock_service(repo.clone())
            .body(json!({ "domain": "^.*example.com$" }))
            .expect_json(json!({ "status": "success" }))
            .test();

        assert!(verify(
            repo.add
                .was_called_with((List::Regex, "^.*example.com$".to_owned()))
        ));
    }
}
