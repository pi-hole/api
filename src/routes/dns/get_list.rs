// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Reading Domain Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    lists::{List, ListServiceGuard},
    util::{reply_result, Reply}
};

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(service: ListServiceGuard) -> Reply {
    reply_result(service.get(List::White))
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(service: ListServiceGuard) -> Reply {
    reply_result(service.get(List::Black))
}

/// Get the Regex list domains
#[get("/dns/regexlist")]
pub fn get_regexlist(service: ListServiceGuard) -> Reply {
    reply_result(service.get(List::Regex))
}

#[cfg(test)]
mod test {
    use crate::{
        lists::{List, ListServiceMock},
        testing::TestBuilder
    };
    use mock_it::verify;

    /// Test that the domains are returned correctly
    fn get_test(list: List, endpoint: &str, domains: Vec<String>) {
        let service = ListServiceMock::new();

        service.get.given(list).will_return(Ok(domains.clone()));

        TestBuilder::new()
            .endpoint(endpoint)
            .mock_service(service.clone())
            .expect_json(json!(domains))
            .test();

        assert!(verify(service.get.was_called_with(list)));
    }

    #[test]
    fn test_get_whitelist() {
        get_test(
            List::White,
            "/admin/api/dns/whitelist",
            vec!["example.com".to_owned(), "example.net".to_owned()]
        );
    }

    #[test]
    fn test_get_blacklist() {
        get_test(
            List::Black,
            "/admin/api/dns/blacklist",
            vec!["example.com".to_owned(), "example.net".to_owned()]
        );
    }

    #[test]
    fn test_get_regexlist() {
        get_test(
            List::Regex,
            "/admin/api/dns/regexlist",
            vec!["^.*example.com$".to_owned(), "example.net".to_owned()]
        );
    }
}
