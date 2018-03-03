/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query Types API Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util;

/// Get the query types
#[get("/stats/query_types")]
pub fn query_types(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("querytypes")?;

    let ipv4 = con.read_f32()?;
    let ipv6 = con.read_f32()?;
    con.expect_eom()?;

    util::reply_data(json!({
        "A": ipv4,
        "AAAA": ipv6
    }))
}
