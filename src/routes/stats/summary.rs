/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Summary Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util::{Reply, reply_data};

/// Get the summary data
#[get("/stats/summary")]
pub fn get_summary(ftl: State<FtlConnectionType>) -> Reply {
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
    let status = match con.read_u8()? {
        0 => "disabled",
        1 => "enabled",
        _ => "unknown"
    };
    con.expect_eom()?;

    reply_data(json!({
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

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_summary() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, -1).unwrap();
        encode::write_i32(&mut data, 7).unwrap();
        encode::write_i32(&mut data, 2).unwrap();
        encode::write_f32(&mut data, 28.571428298950197).unwrap();
        encode::write_i32(&mut data, 6).unwrap();
        encode::write_i32(&mut data, 3).unwrap();
        encode::write_i32(&mut data, 2).unwrap();
        encode::write_i32(&mut data, 3).unwrap();
        encode::write_i32(&mut data, 3).unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/summary")
            .ftl("stats", data)
            .expect_json(
                json!({
                    "domains_blocked": -1,
                    "total_queries": 7,
                    "blocked_queries": 2,
                    "percent_blocked": 28.571428298950197,
                    "unique_domains": 6,
                    "forwarded_queries": 3,
                    "cached_queries": 2,
                    "total_clients": 3,
                    "unique_clients": 3,
                    "status": "unknown"
                })
            )
            .test();
    }
}
