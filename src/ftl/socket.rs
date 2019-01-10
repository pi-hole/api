// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Socket Communication
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::util::{Error, ErrorKind};
use failure::{Fail, ResultExt};
use rmp::{
    decode::{self, DecodeStringError, ValueReadError},
    Marker
};
use std::{
    io::{prelude::*, BufReader},
    os::unix::net::UnixStream
};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::io::Cursor;

/// The location of the FTL socket
const SOCKET_LOCATION: &str = "/var/run/pihole/FTL.sock";

/// A wrapper around the FTL socket to easily read in data. It takes a
/// Box<Read> so that it can be tested with fake data from a Vec<u8>
pub struct FtlConnection<'test>(Box<Read + 'test>);

/// A marker for the type of FTL connection to make.
///
/// - Socket refers to the normal Unix socket connection.
/// - Test is for testing, so that a test can pass in arbitrary MessagePack
/// data to be processed.   The map in Test maps FTL commands to data.
pub enum FtlConnectionType {
    Socket,
    #[cfg(test)]
    Test(HashMap<String, Vec<u8>>)
}

impl FtlConnectionType {
    /// Connect to FTL and run the specified command
    pub fn connect(&self, command: &str) -> Result<FtlConnection, Error> {
        // Determine the type of connection to create
        match *self {
            FtlConnectionType::Socket => {
                // Try to connect to FTL
                let mut stream = match UnixStream::connect(SOCKET_LOCATION) {
                    Ok(s) => s,
                    Err(_) => return Err(Error::from(ErrorKind::FtlConnectionFail))
                };

                // Send the command
                stream
                    .write_all(format!(">{}\n", command).as_bytes())
                    .context(ErrorKind::FtlConnectionFail)?;

                // Return the connection so the API can read the response
                Ok(FtlConnection(Box::new(BufReader::new(stream))))
            }
            #[cfg(test)]
            FtlConnectionType::Test(ref map) => {
                // Return a connection reading the testing data
                Ok(FtlConnection(Box::new(Cursor::new(
                    // Try to get the testing data for this command
                    match map.get(command) {
                        Some(data) => data,
                        None => return Err(Error::from(ErrorKind::FtlConnectionFail))
                    }
                ))))
            }
        }
    }
}

impl<'test> FtlConnection<'test> {
    fn handle_eom_value<T>(result: Result<T, ValueReadError>) -> Result<T, Error> {
        result.map_err(|e| {
            if let ValueReadError::TypeMismatch(marker) = e {
                if marker == Marker::Reserved {
                    // Received EOM
                    return Error::from(e.context(ErrorKind::FtlEomError));
                }
            }

            Error::from(e.context(ErrorKind::FtlReadError))
        })
    }

    fn handle_eom_str<T>(result: Result<T, DecodeStringError>) -> Result<T, Error> {
        result.map_err(|e| {
            if let DecodeStringError::TypeMismatch(ref marker) = e {
                if *marker == Marker::Reserved {
                    // Received EOM
                    return Error::from(ErrorKind::FtlEomError);
                }
            }

            Error::from(ErrorKind::FtlReadError)
        })
    }

    /// We expect an end of message (EOM) response when FTL has finished
    /// sending data
    pub fn expect_eom(&mut self) -> Result<(), Error> {
        let mut buffer: [u8; 1] = [0];

        // Read exactly 1 byte
        match self.0.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => return Err(Error::from(e.context(ErrorKind::FtlReadError)))
        }

        // Check if it was the EOM byte
        if buffer[0] != 0xc1 {
            return Err(Error::from(ErrorKind::FtlReadError));
        }

        Ok(())
    }

    /// Read in an i32 (signed int) value
    pub fn read_i32(&mut self) -> Result<i32, Error> {
        FtlConnection::handle_eom_value(decode::read_i32(&mut self.0))
    }

    /// Read in an i64 (signed long int) value
    pub fn read_i64(&mut self) -> Result<i64, Error> {
        FtlConnection::handle_eom_value(decode::read_i64(&mut self.0))
    }

    /// Read in a string using the buffer
    pub fn read_str<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a str, Error> {
        FtlConnection::handle_eom_str(decode::read_str(&mut self.0, buffer))
    }
}
