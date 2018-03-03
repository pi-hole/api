/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Blocking Status Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::io::prelude::*;
use std::fs::File;

use util;

/// Get the DNS blocking status
#[get("/dns/status")]
pub fn status() -> util::Reply {
    let file = File::open("/etc/dnsmasq.d/01-pihole.conf");

    let status = if file.is_err() {
        // If we failed to open the file, then the status is unknown
        "unknown"
    } else {
        // Read the file to a buffer
        let mut buffer = String::new();
        file?.read_to_string(&mut buffer)?;

        // Check if the gravity.list line is disabled
        let disabled = buffer.lines()
            .filter(|line| *line == "#addn-hosts=/etc/pihole/gravity.list")
            .count();

        // Get the status string
        match disabled {
            0 => "enabled",
            1 => "disabled",
            _ => "unknown"
        }
    };

    util::reply_data(json!({
        "status": status
    }))
}
