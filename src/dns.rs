/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  DNS API Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use util;
use ftl;

use rmp::decode::DecodeStringError;
use rmp::Marker;

fn get_domains(command: &str) -> util::Reply {
    let mut con = ftl::connect(command)?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut domains = Vec::new();

    loop {
        let domain = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_string(),
            Err(e) => {
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

        domains.push(domain);
    }

    util::reply_data(domains)
}

#[get("/dns/whitelist")]
pub fn get_whitelist() -> util::Reply {
    get_domains(">getWhitelist")
}

#[get("/dns/blacklist")]
pub fn get_blacklist() -> util::Reply {
    get_domains(">getBlacklist")
}

#[get("/dns/wildlist")]
pub fn get_wildlist() -> util::Reply {
    get_domains(">getWildlist")
}

#[get("/dns/status")]
pub fn status() -> util::Reply {
    let mut con = ftl::connect(">status")?;

    let status = match con.read_u8()? {
        0 => "disabled",
        1 => "enabled",
        _ => "unknown"
    };
    con.expect_eom()?;

    util::reply_data(json!({
        "status": status
    }))
}
