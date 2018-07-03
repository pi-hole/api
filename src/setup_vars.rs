// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Functions For SetupVars Configuration
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use config::{Env, PiholeFile};
use std::io::BufReader;
use std::io::prelude::*;
use util::Error;

/// Read in a value from setupVars.conf
pub fn read_setup_vars(entry: &str, env: &Env) -> Result<Option<String>, Error> {
    // Open setupVars.conf
    let reader = BufReader::new(env.read_file(PiholeFile::SetupVars)?);

    // Check every line for the key (filter out lines which could not be read)
    for line in reader.lines().filter_map(|item| item.ok()) {
        // Ignore lines without the entry as a substring
        if !line.contains(entry) {
            continue;
        }

        let mut split = line.split("=");

        // Check if we found the key by checking if the line starts with `entry=`
        if split.next().map_or(false, |section| section == entry) {
            return Ok(
                // Get the right hand side if it exists and is not empty
                split
                    .next()
                    .and_then(|item| if item.len() == 0 { None } else { Some(item) })
                    .map(|item| item.to_owned())
            )
        }
    }

    Ok(None)
}
