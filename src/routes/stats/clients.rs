// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Clients Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use ftl::FtlConnectionType;
use rocket::State;
use util::{reply_data, reply_error, Error, ErrorKind, Reply};

/// Get the names of clients
#[get("/stats/clients")]
pub fn clients(_auth: User, ftl: State<FtlConnectionType>) -> Reply {
    match get_clients(&ftl) {
        Ok(data) => reply_data(data),
        Err(err) => reply_error(err)
    }
}

pub fn get_clients(ftl: &FtlConnectionType) -> Result<Vec<Client>, Error> {
    let mut con = ftl.connect("client-names")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut client_data = Vec::new();

    loop {
        // Get the hostname, unless we are at the end of the list
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

        client_data.push(Client { name, ip });
    }

    Ok(client_data)
}

#[derive(Serialize)]
pub struct Client {
    name: String,
    ip: String
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{write_eom, TestBuilder};

    #[test]
    fn test_clients() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_str(&mut data, "10.1.1.1").unwrap();
        encode::write_str(&mut data, "").unwrap();
        encode::write_str(&mut data, "10.1.1.2").unwrap();
        encode::write_str(&mut data, "client3").unwrap();
        encode::write_str(&mut data, "10.1.1.3").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/clients")
            .ftl("client-names", data)
            .expect_json(json!([
                { "name": "client1", "ip": "10.1.1.1" },
                { "name": "",        "ip": "10.1.1.2" },
                { "name": "client3", "ip": "10.1.1.3" }
            ]))
            .test();
    }
}
