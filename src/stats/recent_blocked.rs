/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Recent Blocked Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::DecodeStringError;
use rmp::Marker;
use rocket::State;
use util;
use auth::Auth;

/// Get the most recent blocked domain
#[get("/stats/recent_blocked")]
pub fn recent_blocked(_auth: Auth, ftl: State<FtlConnectionType>) -> util::Reply {
    get_recent_blocked(&ftl, 1)
}

/// Get the `num` most recently blocked domains
#[get("/stats/recent_blocked?<params>")]
pub fn recent_blocked_params(_auth: Auth, ftl: State<FtlConnectionType>, params: RecentBlockedParams) -> util::Reply {
    get_recent_blocked(&ftl, params.num)
}

/// Represents the possible GET parameters on `/stats/recent_blocked`
#[derive(FromForm)]
pub struct RecentBlockedParams {
    num: usize
}

/// Get `num`-many most recently blocked domains
pub fn get_recent_blocked(ftl: &FtlConnectionType, num: usize) -> util::Reply {
    let mut con = ftl.connect(&format!("recentBlocked ({})", num))?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut domains = Vec::with_capacity(num);
    let mut less_domains_than_expected = false;

    for _ in 0..num {
        // Get the next domain. If FTL returns less than what we asked (there haven't been enough
        // blocked domains), then exit the loop
        let domain = match con.read_str(&mut str_buffer) {
            Ok(domain) => domain.to_owned(),
            Err(e) => {
                // Check if we received the EOM
                if let DecodeStringError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        less_domains_than_expected = true;
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        domains.push(domain);
    }

    // If we got the number of domains we expected, then we still need to read the EOM
    if !less_domains_than_expected {
        con.expect_eom()?;
    }

    util::reply_data(domains)
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_recent_blocked() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "example.com").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/recent_blocked")
            .ftl("recentBlocked (1)", data)
            .expect_json(
                json!({
                    "data": [
                        "example.com"
                    ],
                    "errors": []
                })
            )
            .test();
    }

    #[test]
    fn test_recent_blocked_params() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "doubleclick.com").unwrap();
        encode::write_str(&mut data, "google.com").unwrap();
        encode::write_str(&mut data, "ads.net").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/recent_blocked?num=4")
            .ftl("recentBlocked (4)", data)
            .expect_json(
                json!({
                    "data": [
                        "example.com",
                        "doubleclick.com",
                        "google.com",
                        "ads.net"
                    ],
                    "errors": []
                })
            )
            .test();
    }
}
