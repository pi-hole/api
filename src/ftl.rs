use std::os::unix::net::UnixStream;
use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;
use rmp::decode;

const SOCKET_LOCATION: &'static str = "/var/run/pihole/FTL.sock";

pub struct FtlConnection(BufReader<UnixStream>);

pub fn connect(command: &str) -> FtlConnection {
    let mut stream = UnixStream::connect(SOCKET_LOCATION).unwrap();
    stream.write_all(format!(">{}\n", command).as_bytes()).unwrap();

    FtlConnection(BufReader::new(stream))
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

    pub fn read_i32(&mut self) -> Result<i32, decode::ValueReadError> {
        decode::read_i32(&mut self.0)
    }

    pub fn read_f32(&mut self) -> Result<f32, decode::ValueReadError> {
        decode::read_f32(&mut self.0)
    }

    pub fn read_u8(&mut self) -> Result<u8, decode::ValueReadError> {
        decode::read_u8(&mut self.0)
    }
}

