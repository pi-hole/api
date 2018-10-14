// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Blocking Status Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::Env;
use rocket::State;
use settings::{ConfigEntry, SetupVarsEntry};
use util::{reply_data, Reply};

/// Get the DNS blocking status
#[get("/dns/status")]
pub fn status(env: State<Env>) -> Reply {
    let status = if SetupVarsEntry::BlockingEnabled.read_as(&env)? {
        "enabled"
    } else {
        "disabled"
    };

    reply_data(json!({ "status": status }))
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use testing::TestBuilder;

    #[test]
    fn test_status_enabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=true")
            .expect_json(json!({ "status": "enabled" }))
            .test();
    }

    #[test]
    fn test_status_disabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=false")
            .expect_json(json!({ "status": "disabled" }))
            .test();
    }

    #[test]
    fn test_status_default() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "")
            .expect_json(json!({ "status": "enabled" }))
            .test();
    }
}
