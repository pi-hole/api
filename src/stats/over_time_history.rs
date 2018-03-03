/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query History Over Time API Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util;

/// Get the query history over time (separated into blocked and not blocked)
#[get("/stats/overTime/history")]
pub fn over_time_history(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("overTime")?;

    let domains_over_time = con.read_int_map()?;
    let blocked_over_time = con.read_int_map()?;

    util::reply_data(json!({
        "domains_over_time": domains_over_time,
        "blocked_over_time": blocked_over_time
    }))
}
