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
use std::io::BufReader;
use std::collections::HashMap;
use rmp::decode;
use util;

const SOCKET_LOCATION: &'static str = "/var/run/pihole/FTL.sock";

pub struct FtlConnection(BufReader<UnixStream>);

pub fn connect(command: &str) -> Result<FtlConnection, util::Error> {
    let mut stream = match UnixStream::connect(SOCKET_LOCATION) {
        Ok(s) => s,
        Err(_) => return Err(util::Error::FtlConnectionFail)
    };
    stream.write_all(format!(">{}\n", command).as_bytes())?;

    Ok(FtlConnection(BufReader::new(stream)))
}

impl FtlConnection {
    pub fn expect_eom(&mut self) -> Result<(), String> {
        let mut buffer: [u8; 1] = [0];

        match self.0.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => return Err(e.description().to_string())
        }

        if buffer[0] != 0xc1 {
            return Err(format!("Expected EOM (0xc1), got {:2x}", buffer[0]));
        }

        Ok(())
    }

    pub fn read_bool(&mut self) -> Result<bool, decode::ValueReadError> {
        decode::read_bool(&mut self.0)
    }

    pub fn read_u8(&mut self) -> Result<u8, decode::ValueReadError> {
        decode::read_u8(&mut self.0)
    }

    pub fn read_i32(&mut self) -> Result<i32, decode::ValueReadError> {
        decode::read_i32(&mut self.0)
    }

    pub fn read_f32(&mut self) -> Result<f32, decode::ValueReadError> {
        decode::read_f32(&mut self.0)
    }

    pub fn read_str<'r>(&mut self, buffer: &'r mut [u8])
            -> Result<&'r str, decode::DecodeStringError<'r>> {
        decode::read_str(&mut self.0, buffer)
    }

    pub fn read_map_len(&mut self) -> Result<u32, decode::ValueReadError> {
        decode::read_map_len(&mut self.0)
    }

    pub fn read_int_map(&mut self) -> Result<HashMap<i32, i32>, decode::ValueReadError> {
        let map_len = self.read_map_len()? as usize;
        let mut map: HashMap<i32, i32> = HashMap::with_capacity(map_len);

        for _ in 0..map_len {
            let key = self.read_i32()?;
            let value = self.read_i32()?;
            map.insert(key, value);
        }

        Ok(map)
    }
}

