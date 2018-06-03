/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for reading domain lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Config, PiholeFile};
use dns::list::get_list;
use rocket::State;
use util;

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(config: State<Config>) -> util::Reply {
    util::reply_data(get_list(PiholeFile::Whitelist, &config)?)
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(config: State<Config>) -> util::Reply {
    util::reply_data(get_list(PiholeFile::Blacklist, &config)?)
}

/// Get the Regex list domains
#[get("/dns/regexlist")]
pub fn get_regexlist(config: State<Config>) -> util::Reply {
    util::reply_data(get_list(PiholeFile::Regexlist, &config)?)
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use testing::TestBuilder;

    #[test]
    fn test_get_whitelist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/whitelist")
            .file(PiholeFile::Whitelist, "example.com\nexample.net\n")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .expect_json(
                json!([
                    "example.com",
                    "example.net"
                ])
            )
            .test();
    }

    #[test]
    fn test_get_blacklist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/blacklist")
            .file(PiholeFile::Blacklist, "example.com\nexample.net\n")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .expect_json(
                json!([
                    "example.com",
                    "example.net"
                ])
            )
            .test();
    }

    #[test]
    fn test_get_regexlist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/regexlist")
            .file(PiholeFile::Regexlist, "^.*example.com$\nexample.net\n")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .expect_json(
                json!([
                    "^.*example.com$",
                    "example.net"
                ])
            )
            .test();
    }
}
