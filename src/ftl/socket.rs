// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Socket Communication
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use failure::{Fail, ResultExt};
use rmp::{
    decode::{self, DecodeStringError, ValueReadError},
    Marker
};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{BufReader, Cursor};
use std::os::unix::net::UnixStream;
use util::{Error, ErrorKind};

/// The location of the FTL socket
const SOCKET_LOCATION: &'static str = "/var/run/pihole/FTL.sock";

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
                    Err(_) => return Err(ErrorKind::FtlConnectionFail.into())
                };

                // Send the command
                stream
                    .write_all(format!(">{}\n", command).as_bytes())
                    .context(ErrorKind::FtlConnectionFail)?;

                // Return the connection so the API can read the response
                Ok(FtlConnection(Box::new(BufReader::new(stream))))
            }
            FtlConnectionType::Test(ref map) => {
                // Return a connection reading the testing data
                Ok(FtlConnection(Box::new(Cursor::new(
                    // Try to get the testing data for this command
                    match map.get(command) {
                        Some(data) => data,
                        None => return Err(ErrorKind::FtlConnectionFail.into())
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
                    return e.context(ErrorKind::FtlEomError).into();
                }
            }

            e.context(ErrorKind::FtlReadError).into()
        })
    }

    fn handle_eom_str<T>(result: Result<T, DecodeStringError>) -> Result<T, Error> {
        result.map_err(|e| {
            if let DecodeStringError::TypeMismatch(ref marker) = e {
                if *marker == Marker::Reserved {
                    // Received EOM
                    return ErrorKind::FtlEomError.into();
                }
            }

            ErrorKind::FtlReadError.into()
        })
    }

    /// We expect an end of message (EOM) response when FTL has finished
    /// sending data
    pub fn expect_eom(&mut self) -> Result<(), Error> {
        let mut buffer: [u8; 1] = [0];

        // Read exactly 1 byte
        match self.0.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => return Err(e.context(ErrorKind::FtlReadError).into())
        }

        // Check if it was the EOM byte
        if buffer[0] != 0xc1 {
            return Err(ErrorKind::FtlReadError.into());
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

    /// Read in the length of the upcoming map (unsigned int)
    pub fn read_map_len(&mut self) -> Result<u32, Error> {
        FtlConnection::handle_eom_value(decode::read_map_len(&mut self.0))
    }
}
