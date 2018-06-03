/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for adding domains to lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Config, PiholeFile};
use dns::common::reload_gravity;
use dns::list::{add_list, try_remove_list};
use rocket::State;
use rocket_contrib::Json;
use util;
use auth::User;
use ftl::FtlConnectionType;

/// Represents an API input containing a domain
#[derive(Deserialize)]
pub struct DomainInput {
    domain: String
}

/// Add a domain to the whitelist
#[post("/dns/whitelist", data = "<domain_input>")]
pub fn add_whitelist(_auth: User, config: State<Config>, domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the whitelist and remove it from the other lists
    add_list(PiholeFile::Whitelist, domain, &config)?;
    try_remove_list(PiholeFile::Blacklist, domain, &config)?;
    try_remove_list(PiholeFile::Regexlist, domain, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity
    reload_gravity(PiholeFile::Whitelist, &config)?;
    util::reply_success()
}

/// Add a domain to the blacklist
#[post("/dns/blacklist", data = "<domain_input>")]
pub fn add_blacklist(_auth: User, config: State<Config>, domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the blacklist and remove it from the other lists
    add_list(PiholeFile::Blacklist, domain, &config)?;
    try_remove_list(PiholeFile::Whitelist, domain, &config)?;
    try_remove_list(PiholeFile::Regexlist, domain, &config)?;

    // At this point, since we haven't hit an error yet, reload gravity
    reload_gravity(PiholeFile::Blacklist, &config)?;
    util::reply_success()
}

/// Add a domain to the regex list
#[post("/dns/regexlist", data = "<domain_input>")]
pub fn add_regexlist(_auth: User, config: State<Config>, ftl: State<FtlConnectionType>, domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We only need to add it to the regex list
    add_list(PiholeFile::Regexlist, domain, &config)?;

    // At this point, since we haven't hit an error yet, tell FTL to recompile regex
    ftl.connect("recompile-regex")?.expect_eom()?;
    util::reply_success()
}

#[cfg(test)]
mod test {
    use rocket::http::Method;
    use testing::{TestBuilder, write_eom};
    use config::PiholeFile;

    #[test]
    fn test_add_whitelist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/whitelist")
            .method(Method::Post)
            .file_expect(PiholeFile::Whitelist, "", "example.com\n")
            .file(PiholeFile::Blacklist, "")
            .file(PiholeFile::Regexlist, "")
            .file(PiholeFile::SetupVars, "")
            .body(
                json!({
                    "domain": "example.com"
                })
            )
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }

    #[test]
    fn test_add_blacklist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/blacklist")
            .method(Method::Post)
            .file_expect(PiholeFile::Blacklist, "", "example.com\n")
            .file(PiholeFile::Whitelist, "")
            .file(PiholeFile::Regexlist, "")
            .file(PiholeFile::SetupVars, "")
            .body(
                json!({
                    "domain": "example.com"
                })
            )
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }

    #[test]
    fn test_add_regexlist() {
        let mut data = Vec::new();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/dns/regexlist")
            .method(Method::Post)
            .ftl("recompile-regex", data)
            .file_expect(
                PiholeFile::Regexlist,
                "",
                "^.*example.com$\n"
            )
            .file(PiholeFile::Whitelist, "")
            .file(PiholeFile::Blacklist, "")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .body(
                json!({
                    "domain": "^.*example.com$"
                })
            )
            .expect_json(
                json!({
                    "status": "success"
                })
            )
            .test();
    }
}
