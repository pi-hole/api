// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// IO Functions For SetupVars & FTL Configuration Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::{Env, PiholeFile};
use failure::ResultExt;
use settings::entries::{FtlConfEntry, SetupVarsEntry};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use util::{Error, ErrorKind};

/// Read in a value from setupVars.conf
pub fn read_setup_vars(entry: SetupVarsEntry, env: &Env) -> Result<Option<String>, Error> {
    read_setup_file(&entry.key(), env, PiholeFile::SetupVars)
}

/// Read in a value from pihole-FTL.conf
pub fn read_ftl_conf(entry: FtlConfEntry, env: &Env) -> Result<Option<String>, Error> {
    read_setup_file(entry.key(), env, PiholeFile::FtlConfig)
}

/// Write a value to setupVars.conf
pub fn write_setup_vars(entry: SetupVarsEntry, value: &str, env: &Env) -> Result<(), Error> {
    if entry.is_valid(value) {
        write_setup_file(&entry.key(), value, env, PiholeFile::SetupVars)
    } else {
        Err(ErrorKind::InvalidSettingValue.into())
    }
}

/// Write a value to pihole-FTL.conf
pub fn write_ftl_conf(entry: FtlConfEntry, value: &str, env: &Env) -> Result<(), Error> {
    if entry.is_valid(value) {
        write_setup_file(entry.key(), value, env, PiholeFile::FtlConfig)
    } else {
        Err(ErrorKind::InvalidSettingValue.into())
    }
}

/// Read in a value from specified setup file
fn read_setup_file(entry: &str, env: &Env, file: PiholeFile) -> Result<Option<String>, Error> {
    // Open file
    let reader = BufReader::new(env.read_file(file)?);

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
                split.next().and_then(|item| {
                    if item.is_empty() {
                        None
                    } else {
                        Some(item.to_owned())
                    }
                })
            );
        }
    }

    Ok(None)
}

/// Write a value to specified setup file
fn write_setup_file(entry: &str, setting: &str, env: &Env, file: PiholeFile) -> Result<(), Error> {
    // Read specified file, removing any line matching setting to be written
    let entry_equals = format!("{}=", entry);
    let file_read = BufReader::new(env.read_file(file)?);
    let mut setup_vars: Vec<String> = file_read
        .lines()
        .filter_map(|item| item.ok())
        .filter(|line| !line.starts_with(&entry_equals))
        .collect();

    // Append entry to working copy
    let new_entry = format!("{}={}", entry, setting);
    setup_vars.push(new_entry);

    // Open setupVars.conf to be overwritten
    let mut file_write = BufWriter::new(env.write_file(file, false)?);
    let context = ErrorKind::FileWrite(env.file_location(file).to_owned());
    for line in setup_vars {
        file_write
            .write_all(line.as_bytes())
            .context(context.clone().into())?;
        file_write.write_all(b"\n").context(context.clone().into())?;
    }

    file_write.flush().context(context.into())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_setup_vars;
    use env::{Config, Env, PiholeFile};
    use settings::entries::SetupVarsEntry;
    use testing::TestEnvBuilder;

    /// Test to make sure when writing a setting, a similar setting does not
    /// get deleted. Example: Adding PIHOLE_DNS_1 should not delete
    /// PIHOLE_DNS_10
    #[test]
    fn write_similar_keys() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_10=1.1.1.1\n",
            "PIHOLE_DNS_10=1.1.1.1\n\
             PIHOLE_DNS_1=2.2.2.2\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        write_setup_vars(SetupVarsEntry::PiholeDns(1), "2.2.2.2", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_null_value() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=1.2.3.4\n",
            "PIHOLE_DNS_1=\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        write_setup_vars(SetupVarsEntry::PiholeDns(1), "", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_over_duplicate_keys() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=2.2.2.2\n\
             PIHOLE_DNS_1=1.2.3.4\n",
            "PIHOLE_DNS_1=5.6.7.8\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        write_setup_vars(SetupVarsEntry::PiholeDns(1), "5.6.7.8", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_over_null_value() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=\n",
            "PIHOLE_DNS_1=1.2.3.4\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        write_setup_vars(SetupVarsEntry::PiholeDns(1), "1.2.3.4", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_to_empty_file() {
        let env_builder =
            TestEnvBuilder::new().file_expect(PiholeFile::SetupVars, "", "PIHOLE_DNS_1=1.1.1.1\n");
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        write_setup_vars(SetupVarsEntry::PiholeDns(1), "1.1.1.1", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }
}
