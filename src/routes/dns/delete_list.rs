// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Endpoints For Removing Domains From Lists
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use config::Env;
use ftl::FtlConnectionType;
use rocket::State;
use routes::dns::common::reload_gravity;
use routes::dns::list::List;
use util::{reply_success, Reply};

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(_auth: User, env: State<Env>, domain: String) -> Reply {
    List::White.remove(&domain, &env)?;
    reload_gravity(List::White, &env)?;
    reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(_auth: User, env: State<Env>, domain: String) -> Reply {
    List::Black.remove(&domain, &env)?;
    reload_gravity(List::Black, &env)?;
    reply_success()
}

/// Delete a domain from the regex list
#[delete("/dns/regexlist/<domain>")]
pub fn delete_regexlist(
    _auth: User,
    env: State<Env>,
    ftl: State<FtlConnectionType>,
    domain: String
) -> Reply {
    List::Regex.remove(&domain, &env)?;
    ftl.connect("recompile-regex")?.expect_eom()?;
    reply_success()
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use rocket::http::Method;
    use testing::{write_eom, TestBuilder};

    #[test]
    fn test_delete_whitelist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/whitelist/example.com")
            .method(Method::Delete)
            .file_expect(PiholeFile::Whitelist, "example.com\n", "")
            .expect_json(json!({
                    "status": "success"
                }))
            .test();
    }

    #[test]
    fn test_delete_blacklist() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/blacklist/example.com")
            .method(Method::Delete)
            .file_expect(PiholeFile::Blacklist, "example.com\n", "")
            .expect_json(json!({
                    "status": "success"
                }))
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
            .expect_json(json!({
                    "status": "success"
                }))
            .test();
    }
}
