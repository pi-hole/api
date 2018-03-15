/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Forward Destinations Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::DecodeStringError;
use rmp::Marker;
use rocket::State;
use util;

/// Get the forward destinations
#[get("/stats/forward_destinations")]
pub fn forward_destinations(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("forward-dest")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut forward_destinations = Vec::new();

    loop {
        // Read in the hostname, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_string(),
            Err(e) => {
                // Check if we received the EOM
                if let DecodeStringError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        let ip = con.read_str(&mut str_buffer)?;
        let percent = con.read_f32()?;

        // The key will be `hostname|IP` if the hostname exists, otherwise just the IP address
        let key = if ip.len() > 0 {
            format!("{}|{}", name, ip)
        } else {
            name
        };

        forward_destinations.push(ForwardDestination { name: key, percent });
    }

    util::reply_data(forward_destinations)
}

#[derive(Serialize)]
struct ForwardDestination {
    name: String,
    percent: f32
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_forward_destinations() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "google-dns-alt").unwrap();
        encode::write_str(&mut data, "8.8.4.4").unwrap();
        encode::write_f32(&mut data, 0.4).unwrap();
        encode::write_str(&mut data, "google-dns").unwrap();
        encode::write_str(&mut data, "8.8.8.8").unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        encode::write_str(&mut data, "local").unwrap();
        encode::write_str(&mut data, "local").unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/forward_destinations")
            .ftl("forward-dest", data)
            .expect_json(
                json!([
                    {
                        "name": "google-dns-alt|8.8.4.4",
                        "percent": 0.4000000059604645
                    },
                    {
                        "name": "google-dns|8.8.8.8",
                        "percent": 0.30000001192092898
                    },
                    {
                        "name": "local|local",
                        "percent": 0.30000001192092898
                    }
                ])
            )
            .test();
    }
}
