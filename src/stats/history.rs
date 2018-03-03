/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query History Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::ValueReadError;
use rmp::Marker;
use rocket::State;
use util;

/// Get the entire query history (as stored in FTL)
#[get("/stats/history")]
pub fn history(ftl: State<FtlConnectionType>) -> util::Reply {
    get_history(&ftl, "getallqueries")
}

/// Get the query history within the specified timespan
#[get("/stats/history?<timespan>")]
pub fn history_timespan(ftl: State<FtlConnectionType>, timespan: Timespan) -> util::Reply {
    get_history(&ftl, &format!("getallqueries-time {} {}", timespan.from, timespan.to))
}

/// Represents a query returned in `/stats/history`
#[derive(Serialize)]
struct Query(i32, String, String, String, u8, u8);

/// Represents the possible GET parameters on `/stats/history`
#[derive(FromForm)]
pub struct Timespan {
    from: u64,
    to: u64
}

/// Get the query history according to the specified command
fn get_history(ftl: &FtlConnectionType, command: &str) -> util::Reply {
    let mut con = ftl.connect(command)?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut history: Vec<Query> = Vec::new();

    loop {
        // Get the timestamp, unless we are at the end of the list
        let timestamp = match con.read_i32() {
            Ok(timestamp) => timestamp,
            Err(e) => {
                // Check if we received the EOM
                if let ValueReadError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        // Get the rest of the query data
        let query_type = con.read_str(&mut str_buffer)?.to_owned();
        let domain = con.read_str(&mut str_buffer)?.to_owned();
        let client = con.read_str(&mut str_buffer)?.to_owned();
        let status = con.read_u8()?;
        let dnssec = con.read_u8()?;

        history.push(Query(timestamp, query_type, domain, client, status, dnssec));
    }

    util::reply_data(history)
}
