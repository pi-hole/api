/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for reading domain lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::Config;
use dns::list::{get_list, List};
use rocket::State;
use util;

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(config: State<Config>) -> util::Reply {
    get_list(List::Whitelist, &config)
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(config: State<Config>) -> util::Reply {
    get_list(List::Blacklist, &config)
}

/// Get the Wildcard list domains
#[get("/dns/wildlist")]
pub fn get_wildlist(config: State<Config>) -> util::Reply {
    get_list(List::Wildlist, &config)
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use rocket::http::Method;
    use serde_json::Value;
    use testing::test_endpoint_config_multi;

    // Generic test for get_list functions
    fn test_list(
        list_file: PiholeFile,
        initial_content: &str,
        endpoint: &str,
        expected_json: Value
    ) {
        let initial_setup_vars = "IPV4_ADDRESS=10.1.1.1";

        test_endpoint_config_multi(
            Method::Get,
            endpoint,
            vec![
                (PiholeFile::SetupVars, initial_setup_vars, initial_setup_vars),
                (list_file, initial_content, initial_content)
            ],
            expected_json
        );
    }

    #[test]
    fn test_get_whitelist() {
        test_list(
            PiholeFile::Whitelist,
            "example.com\nexample.net\n",
            "/admin/api/dns/whitelist",
            json!({
                "data": [
                    "example.com",
                    "example.net"
                ],
                "errors": []
            })
        );
    }

    #[test]
    fn test_get_blacklist() {
        test_list(
            PiholeFile::Blacklist,
            "example.com\nexample.net\n",
            "/admin/api/dns/blacklist",
            json!({
                "data": [
                    "example.com",
                    "example.net"
                ],
                "errors": []
            })
        );
    }

    #[test]
    fn test_get_wildlist() {
        test_list(
            PiholeFile::Wildlist,
            "address=/example.com/10.1.1.1\naddress=/example.net/10.1.1.1\n",
            "/admin/api/dns/wildlist",
            json!({
                "data": [
                    "example.com",
                    "example.net"
                ],
                "errors": []
            })
        );
    }
}
