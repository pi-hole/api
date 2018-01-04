use std::os::unix::net::UnixStream;
use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;

const SOCKET_LOCATION: &'static str = "/var/run/pihole/FTL.sock";

pub fn connect(command: &str) -> BufReader<UnixStream> {
    let mut stream = UnixStream::connect(SOCKET_LOCATION).unwrap();
    stream.write_all(format!(">{}\n", command).as_bytes()).unwrap();

    BufReader::new(stream)
}

pub fn expect_eom<T: Read>(stream: &mut T) -> Result<(), String> {
    let mut buffer: [u8; 1] = [0];

    match stream.read_exact(&mut buffer) {
        Ok(_) => (),
        Err(e) => return Err(e.description().to_string())
    }

    if buffer[0] != 0xc1 {
        return Err(format!("Expected EOM (0xc1), got {:2x}", buffer[0]));
    }

    Ok(())
}
