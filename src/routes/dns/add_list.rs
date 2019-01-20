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
    routes::{
        auth::User,
        dns::{common::reload_gravity, list::List}
    },
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
pub fn add_whitelist(_auth: User, env: State<Env>, domain_input: Json<DomainInput>) -> Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the whitelist and remove it from the blacklist
    List::White.add(domain, &env)?;
    List::Black.try_remove(domain, &env)?;

    // At this point, since we haven't hit an error yet, reload gravity
    reload_gravity(List::White, &env)?;
    reply_success()
}

/// Add a domain to the blacklist
#[post("/dns/blacklist", data = "<domain_input>")]
pub fn add_blacklist(_auth: User, env: State<Env>, domain_input: Json<DomainInput>) -> Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the blacklist and remove it from the whitelist
    List::Black.add(domain, &env)?;
    List::White.try_remove(domain, &env)?;

    // At this point, since we haven't hit an error yet, reload gravity
    reload_gravity(List::Black, &env)?;
    reply_success()
}

/// Add a domain to the regex list
#[post("/dns/regexlist", data = "<domain_input>")]
pub fn add_regexlist(
    _auth: User,
    env: State<Env>,
    ftl: State<FtlConnectionType>,
    domain_input: Json<DomainInput>
) -> Reply {
    let domain = &domain_input.0.domain;

    // We only need to add it to the regex list
    List::Regex.add(domain, &env)?;

    // At this point, since we haven't hit an error yet, tell FTL to recompile regex
    ftl.connect("recompile-regex")?.expect_eom()?;
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        testing::{write_eom, TestBuilder}
    };
    use rocket::http::Method;

    #[test]
    fn test_add_whitelist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/whitelist")
            .method(Method::Post)
            .file_expect(PiholeFile::Whitelist, "", "example.com\n")
            .file(PiholeFile::Blacklist, "")
            .file(PiholeFile::Regexlist, "")
            .file(PiholeFile::SetupVars, "")
            .body(json!({ "domain": "example.com" }))
            .expect_json(json!({ "status": "success" }))
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
            .body(json!({ "domain": "example.com" }))
            .expect_json(json!({ "status": "success" }))
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
            .file_expect(PiholeFile::Regexlist, "", "^.*example.com$\n")
            .file(PiholeFile::Whitelist, "")
            .file(PiholeFile::Blacklist, "")
            .file(PiholeFile::SetupVars, "IPV4_ADDRESS=10.1.1.1")
            .body(json!({ "domain": "^.*example.com$" }))
            .expect_json(json!({ "status": "success" }))
            .test();
    }
}
