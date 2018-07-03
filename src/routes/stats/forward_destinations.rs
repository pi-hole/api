// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Forward Destinations Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use ftl::FtlConnectionType;
use rocket::State;
use util::{reply_data, ErrorKind, Reply};

/// Get the forward destinations
#[get("/stats/forward_destinations")]
pub fn forward_destinations(_auth: User, ftl: State<FtlConnectionType>) -> Reply {
    let mut con = ftl.connect("forward-dest")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut forward_destinations = Vec::new();

    loop {
        // Read in the hostname, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_owned(),
            Err(e) => {
                // Check if we received the EOM
                if e.kind() == ErrorKind::FtlEomError {
                    break;
                }

                // Unknown read error
                return Err(e);
            }
        };

        let ip = con.read_str(&mut str_buffer)?.to_owned();
        let percent = con.read_f32()?;

        forward_destinations.push(ForwardDestination { name, ip, percent });
    }

    reply_data(forward_destinations)
}

#[derive(Serialize)]
struct ForwardDestination {
    name: String,
    ip: String,
    percent: f32
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{write_eom, TestBuilder};

    #[test]
    fn test_forward_destinations() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "google-dns-alt").unwrap();
        encode::write_str(&mut data, "8.8.4.4").unwrap();
        encode::write_f32(&mut data, 0.4).unwrap();
        encode::write_str(&mut data, "google-dns").unwrap();
        encode::write_str(&mut data, "8.8.8.8").unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        encode::write_str(&mut data, "cache").unwrap();
        encode::write_str(&mut data, "cache").unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/forward_destinations")
            .ftl("forward-dest", data)
            .expect_json(json!([
                    {
                        "name": "google-dns-alt",
                        "ip": "8.8.4.4",
                        "percent": 0.4000000059604645
                    },
                    {
                        "name": "google-dns",
                        "ip": "8.8.8.8",
                        "percent": 0.30000001192092898
                    },
                    {
                        "name": "cache",
                        "ip": "cache",
                        "percent": 0.30000001192092898
                    }
                ]))
            .test();
    }
}
