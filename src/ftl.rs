use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::io::BufReader;

const SOCKET_LOCATION: &'static str = "/var/run/pihole/FTL.sock";

pub struct FtlIter(BufReader<UnixStream>);

pub fn connect(command: &str) -> FtlIter {
    let mut stream = UnixStream::connect(SOCKET_LOCATION).unwrap();
    stream.write_all(format!(">{}\n", command).as_bytes()).unwrap();

    FtlIter(BufReader::new(stream))
}

impl Iterator for FtlIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data = String::new();
        self.0.read_line(&mut data).unwrap();

        if data.contains("---EOM---") {
            return None;
        }

        if data.ends_with("\n") {
            data.pop();
        }

        Some(data)
    }
}
