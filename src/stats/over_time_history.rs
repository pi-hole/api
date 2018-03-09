/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query History Over Time Endpoint
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

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_over_time_history() {
        let mut data = Vec::new();
        encode::write_map_len(&mut data, 2).unwrap();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_i32(&mut data, 10).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_i32(&mut data, 20).unwrap();
        encode::write_map_len(&mut data, 2).unwrap();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_i32(&mut data, 5).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_i32(&mut data, 5).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/history")
            .ftl("overTime", data)
            .expect_json(
                json!({
                    "data": {
                        "blocked_over_time": {
                            "1520126228": 5,
                            "1520126406": 5
                        },
                        "domains_over_time": {
                            "1520126228": 10,
                            "1520126406": 20
                        }
                    },
                    "errors": []
                })
            )
            .test();
    }
}
