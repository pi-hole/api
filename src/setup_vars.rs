/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Functions related to the setupVars.conf file
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Env, PiholeFile};
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use util::{Error};
use setup_validate::validate;

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

/// Write a value to setupVars.conf
#[allow(unused)]
pub fn write_setup_vars(entry: &str, setting: &str, env: &Env) -> Result<Option<String>, Error> {

    // Perform validity check before doing anything with the file
//    if !is_setting_valid(&entry, &setting) { return Err("Invalid setting")};
    if !validate(&entry, &setting) { return Ok(Some("Invalid setting".to_owned())) };    

    // Read setupVars.conf, removing any line matching setting to be written
    let mut setup_vars = Vec::new();
    let file_read = BufReader::new(env.read_file(PiholeFile::SetupVars)?);
    for line in file_read.lines().filter_map(|ln| ln.ok()) {
        if !line.contains(entry) {
            setup_vars.push(format!("{}\n",line));
        }
    }
    // Append entry to working copy
    let new_entry = format!("{}={}\n", &entry, &setting);
    setup_vars.push(new_entry);

    // Open setupVars.conf to be overwritten
    let mut file_write = BufWriter::new(env.write_file(PiholeFile::SetupVars, false)?);
    for line_out in &setup_vars {
      &file_write.write(line_out.as_bytes()).expect("Unable to write data");
    }
    file_write.flush().expect("Unable to close file");
    Ok(None)
}

