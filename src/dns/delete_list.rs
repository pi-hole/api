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
use ftl::FtlConnectionType;

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(_auth: User, config: State<Config>, domain: String) -> util::Reply {
    remove_list(PiholeFile::Whitelist, &domain, &config)?;
    reload_gravity(PiholeFile::Whitelist, &config)?;
    util::reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(_auth: User, config: State<Config>, domain: String) -> util::Reply {
    remove_list(PiholeFile::Blacklist, &domain, &config)?;
    reload_gravity(PiholeFile::Blacklist, &config)?;
    util::reply_success()
}

/// Delete a domain from the regex list
#[delete("/dns/regexlist/<domain>")]
pub fn delete_regexlist(_auth: User, config: State<Config>, ftl: State<FtlConnectionType>, domain: String) -> util::Reply {
    remove_list(PiholeFile::Regexlist, &domain, &config)?;
    ftl.connect("recompile-regex")?.expect_eom()?;
    util::reply_success()
}

#[cfg(test)]
mod test {
    use testing::{TestBuilder, write_eom};
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
    fn test_delete_regexlist() {
        let mut data = Vec::new();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/dns/regexlist/^.*example.com$")
            .method(Method::Delete)
            .ftl("recompile-regex", data)
            .file_expect(PiholeFile::Regexlist, "^.*example.com$\n", "")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }
}
