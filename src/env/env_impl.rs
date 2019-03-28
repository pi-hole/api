// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Environment Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Config, PiholeFile},
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader},
    os::unix::fs::OpenOptionsExt,
    path::Path
};

#[cfg(test)]
use failure::Fail;
#[cfg(test)]
use std::{
    collections::HashMap,
    io::{self, Read, Write}
};
#[cfg(test)]
use tempfile::{tempfile, NamedTempFile};

/// Environment of the Pi-hole API. Stores the config and abstracts away some
/// systems to make testing easier.
pub enum Env {
    Production(Config),
    #[cfg(test)]
    Test(Config, HashMap<PiholeFile, NamedTempFile>)
}

impl Clone for Env {
    fn clone(&self) -> Self {
        match self {
            Env::Production(config) => Env::Production(config.clone()),
            // There is no good way to copy NamedTempFiles, and we shouldn't be
            // doing that during a test anyways
            #[cfg(test)]
            Env::Test(_, _) => unimplemented!()
        }
    }
}

impl Env {
    /// Get the API config that was loaded
    pub fn config(&self) -> &Config {
        match self {
            Env::Production(config) => config,
            #[cfg(test)]
            Env::Test(config, _) => config
        }
    }

    /// Get the location of a file
    pub fn file_location(&self, file: PiholeFile) -> &str {
        match self {
            Env::Production(config) => config.file_location(file),
            #[cfg(test)]
            Env::Test(_, _) => file.default_location()
        }
    }

    /// Open a file for reading
    pub fn read_file(&self, file: PiholeFile) -> Result<File, Error> {
        match self {
            Env::Production(_) => {
                let file_location = self.file_location(file);
                File::open(file_location)
                    .context(ErrorKind::FileRead(file_location.to_owned()))
                    .map_err(Error::from)
            }
            #[cfg(test)]
            Env::Test(_, map) => match map.get(&file) {
                Some(file) => file
                    .reopen()
                    .context(ErrorKind::Unknown)
                    .map_err(Error::from),
                // Return a NotFound error, wrapped in a FileRead error
                None => Err(Error::from(
                    io::Error::from(io::ErrorKind::NotFound)
                        .context(ErrorKind::FileRead(self.file_location(file).to_owned()))
                ))
            }
        }
    }

    /// Open a file and read its lines. This uses a `BufReader` under the hood
    /// and skips lines with errors (invalid UTF-8).
    pub fn read_file_lines(&self, file: PiholeFile) -> Result<Vec<String>, Error> {
        let reader = BufReader::new(self.read_file(file)?);
        Ok(reader.lines().filter_map(Result::ok).collect())
    }

    /// Open a file for writing. If `append` is false, the file will be
    /// truncated.
    pub fn write_file(&self, file: PiholeFile, append: bool) -> Result<File, Error> {
        match self {
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
            #[cfg(test)]
            Env::Test(_, map) => {
                let file = match map.get(&file) {
                    Some(file) => file.reopen().context(ErrorKind::Unknown)?,
                    None => {
                        // Return a NotFound error, wrapped in a FileRead error
                        return Err(Error::from(
                            io::Error::from(io::ErrorKind::NotFound)
                                .context(ErrorKind::FileRead(self.file_location(file).to_owned()))
                        ));
                    }
                };

                if !append {
                    file.set_len(0).context(ErrorKind::Unknown)?;
                }

                Ok(file)
            }
        }
    }

    /// Rename (move) a file from `from` to `to`
    pub fn rename_file(&self, from: PiholeFile, to: PiholeFile) -> Result<(), Error> {
        match self {
            Env::Production(_) => {
                let to_path = self.file_location(to);

                fs::rename(self.file_location(from), to_path)
                    .context(ErrorKind::FileWrite(to_path.to_owned()))?;

                Ok(())
            }
            #[cfg(test)]
            Env::Test(_, ref map) => {
                let mut from_file = match map.get(&from) {
                    Some(file) => file.reopen().context(ErrorKind::Unknown)?,
                    // It's an error if the from file does not exist
                    None => {
                        // Return a NotFound error, wrapped in a FileRead error
                        return Err(Error::from(
                            io::Error::from(io::ErrorKind::NotFound)
                                .context(ErrorKind::FileRead(self.file_location(from).to_owned()))
                        ));
                    }
                };

                let mut to_file = match map.get(&to) {
                    Some(file) => file.reopen().context(ErrorKind::Unknown)?,
                    // It's fine if the to file does not exist, create one
                    None => tempfile().context(ErrorKind::Unknown)?
                };

                // Copy the data from the "from" file to the "to" file.
                // At the end, the "from" file is empty and the "to" file has
                // the original contents of the "from" file.
                let mut buffer = Vec::new();
                from_file
                    .read_to_end(&mut buffer)
                    .context(ErrorKind::Unknown)?;
                to_file.set_len(0).context(ErrorKind::Unknown)?;
                to_file.write_all(&buffer).context(ErrorKind::Unknown)?;
                from_file.set_len(0).context(ErrorKind::Unknown)?;

                Ok(())
            }
        }
    }

    /// Check if a file exists
    pub fn file_exists(&self, file: PiholeFile) -> bool {
        match self {
            Env::Production(_) => Path::new(self.file_location(file)).is_file(),
            #[cfg(test)]
            Env::Test(_, map) => map.contains_key(&file)
        }
    }

    /// Check if we're in a testing environment
    pub fn is_test(&self) -> bool {
        match self {
            Env::Production(_) => false,
            #[cfg(test)]
            Env::Test(_, _) => true
        }
    }
}
