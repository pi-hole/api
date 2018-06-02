/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for removing domains from lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Config, PiholeFile};
use dns::common::reload_gravity;
use dns::list::remove_list;
use rocket::State;
use util;
use auth::User;

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(_auth: User, config: State<Config>, domain: String) -> util::Reply {
    remove_list(PiholeFile::Whitelist, &domain, &config)?;
    reload_gravity(PiholeFile::Whitelist, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(_auth: User, config: State<Config>, domain: String) -> util::Reply {
    remove_list(PiholeFile::Blacklist, &domain, &config)?;
    reload_gravity(PiholeFile::Blacklist, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the wildcard list
#[delete("/dns/wildlist/<domain>")]
pub fn delete_wildlist(_auth: User, config: State<Config>, domain: String) -> util::Reply {
    remove_list(PiholeFile::Wildlist, &domain, &config)?;
    reload_gravity(PiholeFile::Wildlist, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

#[cfg(test)]
mod test {
    use testing::TestBuilder;
    use config::PiholeFile;
    use rocket::http::Method;

    #[test]
    fn test_delete_whitelist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/whitelist/example.com")
            .method(Method::Delete)
            .file_expect(PiholeFile::Whitelist, "example.com\n", "")
            .file(PiholeFile::SetupVars, "")
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }

    #[test]
    fn test_delete_blacklist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/blacklist/example.com")
            .method(Method::Delete)
            .file_expect(PiholeFile::Blacklist, "example.com\n", "")
            .file(PiholeFile::SetupVars, "")
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }

    #[test]
    fn test_delete_wildlist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/wildlist/example.com")
            .method(Method::Delete)
            .file_expect(PiholeFile::Wildlist, "address=/example.com/10.1.1.1\n", "")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }
}
