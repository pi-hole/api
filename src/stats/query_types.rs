/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query Types Endpoint
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

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{test_endpoint, write_eom};

    #[test]
    fn test_query_types() {
        let mut data = Vec::new();
        encode::write_f32(&mut data, 0.7).unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        write_eom(&mut data);

        test_endpoint(
            "/admin/api/stats/query_types",
            "querytypes",
            data,
            json!({
                "data": {
                    "A": 0.699999988079071,
                    "AAAA": 0.30000001192092898
                },
                "errors": []
            })
        );
    }
}
