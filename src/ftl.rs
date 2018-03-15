/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  FTL Communication Utilities
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::os::unix::net::UnixStream;
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufReader, Cursor};
use std::collections::HashMap;
use rmp::decode;

use util;

/// The location of the FTL socket
const SOCKET_LOCATION: &'static str = "/var/run/pihole/FTL.sock";

/// A wrapper around the FTL socket to easily read in data. It takes a Box<Read> so that it can be
/// tested with fake data from a Vec<u8>
pub struct FtlConnection<'test>(Box<Read + 'test>);

/// A marker for the type of FTL connection to make.
///
/// - Socket refers to the normal Unix socket connection.
/// - Test is for testing, so that a test can pass in arbitrary MessagePack data to be processed.
///   The map in Test maps FTL commands to data.
pub enum FtlConnectionType {
    Socket,
    Test(HashMap<String, Vec<u8>>)
}

impl FtlConnectionType {
    /// Connect to FTL and run the specified command
    pub fn connect(&self, command: &str) -> Result<FtlConnection, util::Error> {
        // Determine the type of connection to create
        match *self {
            FtlConnectionType::Socket => {
                // Try to connect to FTL
                let mut stream = match UnixStream::connect(SOCKET_LOCATION) {
                    Ok(s) => s,
                    Err(_) => return Err(util::Error::FtlConnectionFail)
                };

                // Send the command
                stream.write_all(format!(">{}\n", command).as_bytes())?;

                // Return the connection so the API can read the response
                Ok(FtlConnection(Box::new(BufReader::new(stream))))
            },
            FtlConnectionType::Test(ref map) => {
                // Return a connection reading the testing data
                Ok(FtlConnection(Box::new(Cursor::new(
                    // Try to get the testing data for this command
                    match map.get(command) {
                        Some(data) => data,
                        None => return Err(util::Error::FtlConnectionFail)
                    }
                ))))
            }
        }
    }
}

impl<'test> FtlConnection<'test> {
    /// We expect an end of message (EOM) response when FTL has finished sending data
    pub fn expect_eom(&mut self) -> Result<(), String> {
        let mut buffer: [u8; 1] = [0];

        // Read exactly 1 byte
        match self.0.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => return Err(e.description().to_string())
        }

        // Check if it was the EOM byte
        if buffer[0] != 0xc1 {
            return Err(format!("Expected EOM (0xc1), got {:2x}", buffer[0]));
        }

        Ok(())
    }

    /// Read in a bool value
    pub fn read_bool(&mut self) -> Result<bool, decode::ValueReadError> {
        decode::read_bool(&mut self.0)
    }

    /// Read in a u8 (unsigned byte) value
    pub fn read_u8(&mut self) -> Result<u8, decode::ValueReadError> {
        decode::read_u8(&mut self.0)
    }

    /// Read in an i32 (signed int) value
    pub fn read_i32(&mut self) -> Result<i32, decode::ValueReadError> {
        decode::read_i32(&mut self.0)
    }

    /// Read in an f32 (float) value
    pub fn read_f32(&mut self) -> Result<f32, decode::ValueReadError> {
        decode::read_f32(&mut self.0)
    }

    /// Read in a string using the buffer
    pub fn read_str<'r>(&mut self, buffer: &'r mut [u8])
            -> Result<&'r str, decode::DecodeStringError<'r>> {
        decode::read_str(&mut self.0, buffer)
    }

    /// Read in the length of the upcoming map (unsigned int)
    pub fn read_map_len(&mut self) -> Result<u32, decode::ValueReadError> {
        decode::read_map_len(&mut self.0)
    }
}

