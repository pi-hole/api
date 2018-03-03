/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for reading domain lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::io::prelude::*;
use std::io::{self, BufReader};
use std::fs::File;

use util;
use dns::common::read_setup_vars;
use dns::list::List;

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist() -> util::Reply {
    get_list(List::Whitelist)
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist() -> util::Reply {
    get_list(List::Blacklist)
}

/// Get the Wildcard list domains
#[get("/dns/wildlist")]
pub fn get_wildlist() -> util::Reply {
    get_list(List::Wildlist)
}

/// Read in the domains from a list
fn get_list(list: List) -> util::Reply {
    let file = match File::open(list.location()) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                // If the file is not found, then the list is empty. [0; 0] is an empty list of
                // type i32. We can't use [] because the type needs to be known.
                return util::reply_data([0; 0]);
            } else {
                return Err(e.into());
            }
        }
    };

    let reader = BufReader::new(file);

    // Used for the wildcard list to skip IPv6 lines
    let mut skip_lines = false;
    let is_wildcard = list == List::Wildlist;

    if is_wildcard {
        // Check if both IPv4 and IPv6 are used.
        // If so, skip every other line if we're getting wildcard domains.
        let ipv4 = read_setup_vars("IPV4_ADDRESS")?;
        let ipv6 = read_setup_vars("IPV6_ADDRESS")?;

        skip_lines = ipv4.is_some() && ipv6.is_some();
    }

    let mut skip = true;
    let mut lines = Vec::new();

    // Read in the domains
    for line in reader.lines().filter_map(|item| item.ok()) {
        // Skip empty lines
        if line.len() == 0 {
            continue;
        }

        // The wildcard list sometimes skips every other, see above
        if skip_lines {
            skip = !skip;

            if skip {
                continue;
            }
        }

        // Parse the line
        let mut parsed_line = if is_wildcard {
            // If we're reading wildcards, the domain is between two forward slashes
            match line.split("/").nth(1) {
                Some(s) => s.to_owned(),
                None => continue
            }
        } else {
            line
        };

        lines.push(parsed_line);
    }

    util::reply_data(lines)
}
