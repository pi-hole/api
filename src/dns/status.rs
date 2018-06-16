/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Blocking Status Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Env, PiholeFile};
use std::io::{BufRead, BufReader};
use std::fs::File;
use rocket::State;
use util;

/// Get the DNS blocking status
#[get("/dns/status")]
pub fn status(env: State<Env>) -> util::Reply {
    let status = match env.read_file(PiholeFile::DnsmasqMainConfig) {
        Ok(file) => check_for_gravity(file),

        // If we failed to open the file, then the status is unknown
        Err(_) => "unknown"
    };

    util::reply_data(json!({
        "status": status
    }))
}

/// Check a file for the `addn-hosts=/.../gravity.list` line and return the blocking status
fn check_for_gravity(file: File) -> &'static str {
    // Read the file through a buffer
    let reader = BufReader::new(file);

    // Check for the gravity.list line
    for line in reader.lines().filter_map(|item| item.ok()) {
        if line == "#addn-hosts=/etc/pihole/gravity.list" {
            return "disabled";
        } else if line == "addn-hosts=/etc/pihole/gravity.list" {
            return "enabled";
        }
    }

    "unknown"
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use testing::TestBuilder;

    #[test]
    fn test_status_enabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::DnsmasqMainConfig, "addn-hosts=/etc/pihole/gravity.list")
            .expect_json(
                json!({
                    "status": "enabled"
                })
            )
            .test();
    }

    #[test]
    fn test_status_disabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::DnsmasqMainConfig, "#addn-hosts=/etc/pihole/gravity.list")
            .expect_json(
                json!({
                    "status": "disabled"
                })
            )
            .test();
    }

    #[test]
    fn test_status_unknown() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::DnsmasqMainConfig, "random data...")
            .expect_json(
                json!({
                    "status": "unknown"
                })
            )
            .test();
    }
}
