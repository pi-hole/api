// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Functions For SetupVars & FTL Configuration
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use config::{Env, PiholeFile};
use config_files::*;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use util::{Error, ErrorKind};

/// Read in a value from setupVars.conf - retained for dns
pub fn read_setup_vars_dns(entry: &str, env: &Env) -> Result<Option<String>, Error> {
    read_setup_file(&entry, &env, PiholeFile::SetupVars)
}

/// Read in a value from setupVars.conf
pub fn read_setup_vars(entry: SetupVarsEntry, env: &Env) -> Result<Option<String>, Error> {
    read_setup_file(&entry.key(), &env, PiholeFile::SetupVars)
}

/// Read in a value from pihole-FTL.conf
#[allow(dead_code)]
pub fn read_ftl_conf(entry: FTLConfEntry, env: &Env) -> Result<Option<String>, Error> {
    read_setup_file(&entry.key(), &env, PiholeFile::FTLConfig)
}

/// Write a value to setupVars.conf
#[allow(dead_code)]
pub fn write_setup_vars(entry: SetupVarsEntry, setting: &str, env: &Env) -> Result<(), Error> {
    if entry.validate(&setting) {
        return write_setup_file(&entry.key(), &setting, &env, PiholeFile::SetupVars);
    } else {
        return Err(ErrorKind::Unknown.into());
    }
}

/// Write a value to pihole-FTL.conf
#[allow(dead_code)]
pub fn write_ftl_conf(entry: FTLConfEntry, setting: &str, env: &Env) -> Result<(), Error> {
    if entry.validate(&setting) {
        return write_setup_file(&entry.key(), &setting, &env, PiholeFile::FTLConfig);
    } else {
        return Err(ErrorKind::Unknown.into());
    }
}

/// Read in a value from specified setup file
fn read_setup_file(
    entry: &str,
    env: &Env,
    piholesetupfile: PiholeFile
) -> Result<Option<String>, Error> {
    // Open file
    let reader = BufReader::new(env.read_file(piholesetupfile)?);

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
            );
        }
    }
    Ok(None)
}

/// Write a value to specified setup file
#[allow(unused)]
fn write_setup_file(
    entry: &str,
    setting: &str,
    env: &Env,
    piholesetupfile: PiholeFile
) -> Result<(), Error> {
    // Read specified file, removing any line matching setting to be written
    let mut setup_vars = Vec::new();
    let file_read = BufReader::new(env.read_file(piholesetupfile)?);
    setup_vars = file_read
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.contains(entry))
        .collect();
    // Append entry to working copy
    let new_entry = format!("{}={}", &entry, &setting);
    setup_vars.push(new_entry);
    // Open setupVars.conf to be overwritten and write
    let mut file_write = BufWriter::new(env.write_file(piholesetupfile, false)?);
    for line_out in setup_vars {
        match file_write.write(line_out.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                return Err(
                    ErrorKind::FileWrite(piholesetupfile.default_location().to_owned()).into()
                )
            }
        };
        match file_write.write(b"\n") {
            Ok(_) => {}
            Err(_) => {
                return Err(
                    ErrorKind::FileWrite(piholesetupfile.default_location().to_owned()).into()
                )
            }
        };
    }
    match file_write.flush() {
        Ok(_) => Ok(()),
        Err(_) => {
            return Err(ErrorKind::FileWrite(piholesetupfile.default_location().to_owned()).into())
        }
    }
}
