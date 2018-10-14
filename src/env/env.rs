// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Environment Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::{Config, PiholeFile};
use failure::ResultExt;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use tempfile::NamedTempFile;
use util::{Error, ErrorKind};

/// Environment of the Pi-hole API. Stores the config and abstracts away some
/// systems to make testing easier.
pub enum Env {
    Production(Config),
    Test(Config, HashMap<PiholeFile, NamedTempFile>)
}

impl Env {
    pub fn config(&self) -> &Config {
        match *self {
            Env::Production(ref config) => config,
            Env::Test(ref config, _) => config
        }
    }

    /// Get the location of a file
    pub fn file_location(&self, file: PiholeFile) -> &str {
        match *self {
            Env::Production(ref config_options) => config_options.file_location(file),
            Env::Test(_, _) => file.default_location()
        }
    }

    /// Open a file for reading
    pub fn read_file(&self, file: PiholeFile) -> Result<File, Error> {
        match *self {
            Env::Production(_) => {
                let file_location = self.file_location(file);
                File::open(file_location)
                    .context(ErrorKind::FileRead(file_location.to_owned()))
                    .map_err(Error::from)
            }
            Env::Test(_, ref map) => match map.get(&file) {
                Some(data) => data,
                None => return Err(ErrorKind::NotFound.into())
            }.reopen()
                .context(ErrorKind::Unknown)
                .map_err(Error::from)
        }
    }

    /// Open a file for writing. If `append` is false, the file will be
    /// truncated.
    pub fn write_file(&self, file: PiholeFile, append: bool) -> Result<File, Error> {
        match *self {
            Env::Production(_) => {
                let mut open_options = OpenOptions::new();
                open_options.create(true).write(true).mode(0o644);

                if append {
                    open_options.append(true);
                } else {
                    open_options.truncate(true);
                }

                let file_location = self.file_location(file);
                open_options
                    .open(file_location)
                    .context(ErrorKind::FileWrite(file_location.to_owned()))
                    .map_err(Error::from)
            }
            Env::Test(_, ref map) => {
                let file = match map.get(&file) {
                    Some(data) => data,
                    None => return Err(ErrorKind::NotFound.into())
                }.reopen()
                    .context(ErrorKind::Unknown)?;

                if !append {
                    file.set_len(0).context(ErrorKind::Unknown)?;
                }

                Ok(file)
            }
        }
    }

    /// Check if a file exists
    #[allow(unused)]
    pub fn file_exists(&self, file: PiholeFile) -> bool {
        match *self {
            Env::Production(_) => Path::new(self.file_location(file)).is_file(),
            Env::Test(_, ref map) => map.contains_key(&file)
        }
    }

    /// Check if we're in a testing environment
    pub fn is_test(&self) -> bool {
        match *self {
            Env::Production(_) => false,
            Env::Test(_, _) => true
        }
    }
}
