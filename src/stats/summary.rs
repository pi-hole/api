/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Summary API Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util;

/// Get the summary data
#[get("/stats/summary")]
pub fn get_summary(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("stats")?;

    // Read in the data
    let domains_blocked = con.read_i32()?;
    let total_queries = con.read_i32()?;
    let blocked_queries = con.read_i32()?;
    let percent_blocked = con.read_f32()?;
    let unique_domains = con.read_i32()?;
    let forwarded_queries = con.read_i32()?;
    let cached_queries = con.read_i32()?;
    let total_clients = con.read_i32()?;
    let unique_clients = con.read_i32()?;
    let status = con.read_u8()?;
    con.expect_eom()?;

    util::reply_data(json!({
        "domains_blocked": domains_blocked,
        "total_queries": total_queries,
        "blocked_queries": blocked_queries,
        "percent_blocked": percent_blocked,
        "unique_domains": unique_domains,
        "forwarded_queries": forwarded_queries,
        "cached_queries": cached_queries,
        "total_clients": total_clients,
        "unique_clients": unique_clients,
        "status": status
    }))
}
