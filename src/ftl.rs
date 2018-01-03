use std::os::unix::net::UnixStream;

pub fn connect() -> UnixStream {
    UnixStream::connect("/var/run/pihole/FTL.sock").unwrap()
}
