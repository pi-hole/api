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
    lists::{List, ListServiceGuard},
    routes::auth::User,
    util::{reply_success, Reply}
};
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
    list_service: ListServiceGuard,
    domain_input: Json<DomainInput>
) -> Reply {
    list_service.add(List::White, &domain_input.0.domain)?;
    reply_success()
}

/// Add a domain to the blacklist
#[post("/dns/blacklist", data = "<domain_input>")]
pub fn add_blacklist(
    _auth: User,
    list_service: ListServiceGuard,
    domain_input: Json<DomainInput>
) -> Reply {
    list_service.add(List::Black, &domain_input.0.domain)?;
    reply_success()
}

/// Add a domain to the regex list
#[post("/dns/regexlist", data = "<domain_input>")]
pub fn add_regexlist(
    _auth: User,
    list_service: ListServiceGuard,
    domain_input: Json<DomainInput>
) -> Reply {
    list_service.add(List::Regex, &domain_input.0.domain)?;
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::{
        lists::{List, ListServiceMock},
        testing::TestBuilder
    };
    use mock_it::verify;
    use rocket::http::Method;

    /// Test that a successful add returns success
    fn add_test(list: List, endpoint: &str, domain: &str) {
        let service = ListServiceMock::new();

        service
            .add
            .given((list, domain.to_owned()))
            .will_return(Ok(()));

        TestBuilder::new()
            .endpoint(endpoint)
            .method(Method::Post)
            .mock_service(service.clone())
            .body(json!({ "domain": domain }))
            .expect_json(json!({ "status": "success" }))
            .test();

        assert!(verify(
            service.add.was_called_with((list, domain.to_owned()))
        ));
    }

    #[test]
    fn add_whitelist() {
        add_test(List::White, "/admin/api/dns/whitelist", "example.com");
    }

    /// A successful add returns success
    #[test]
    fn add_blacklist() {
        add_test(List::Black, "/admin/api/dns/blacklist", "example.com");
    }

    /// A successful add returns success
    #[test]
    fn test_add_regexlist() {
        add_test(List::Regex, "/admin/api/dns/regexlist", "^.*example.com$");
    }
}
