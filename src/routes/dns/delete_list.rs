// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Removing Domains From Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    routes::auth::User,
    services::lists::{List, ListServiceGuard},
    util::{reply_success, Reply}
};

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(_auth: User, list_service: ListServiceGuard, domain: String) -> Reply {
    list_service.remove(List::White, &domain)?;
    reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(_auth: User, list_service: ListServiceGuard, domain: String) -> Reply {
    list_service.remove(List::Black, &domain)?;
    reply_success()
}

/// Delete a domain from the regex list
#[delete("/dns/regexlist/<domain>")]
pub fn delete_regexlist(_auth: User, list_service: ListServiceGuard, domain: String) -> Reply {
    list_service.remove(List::Regex, &domain)?;
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::{
        services::lists::{List, ListServiceMock},
        testing::TestBuilder
    };
    use mock_it::verify;
    use rocket::http::Method;

    /// Test that a successful delete returns success
    fn delete_test(list: List, endpoint: &str, domain: &str) {
        let service = ListServiceMock::new();

        service
            .remove
            .given((list, domain.to_owned()))
            .will_return(Ok(()));

        TestBuilder::new()
            .endpoint(endpoint)
            .method(Method::Delete)
            .mock_service(service.clone())
            .expect_json(json!({ "status": "success" }))
            .test();

        assert!(verify(
            service.remove.was_called_with((list, domain.to_owned()))
        ));
    }

    #[test]
    fn test_delete_whitelist() {
        delete_test(
            List::White,
            "/admin/api/dns/whitelist/example.com",
            "example.com"
        );
    }

    #[test]
    fn test_delete_blacklist() {
        delete_test(
            List::Black,
            "/admin/api/dns/blacklist/example.com",
            "example.com"
        );
    }

    #[test]
    fn test_delete_regexlist() {
        delete_test(
            List::Regex,
            "/admin/api/dns/regexlist/%5E.%2Aexample.com%24",
            "^.*example.com$"
        );
    }
}
