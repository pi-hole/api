// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query History Over Time Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::{FtlConnectionType, FtlConnection};
use rocket::State;
use util::{Reply, Error, reply_data};

/// Get the query history over time (separated into blocked and not blocked)
#[get("/stats/overTime/history")]
pub fn over_time_history(ftl: State<FtlConnectionType>) -> Reply {
    let mut con = ftl.connect("overTime")?;

    let domains_over_time = get_over_time_data(&mut con)?;
    let blocked_over_time = get_over_time_data(&mut con)?;

    reply_data(json!({
        "domains_over_time": domains_over_time,
        "blocked_over_time": blocked_over_time
    }))
}

/// Read in some time data (represented by FTL as a map of ints to ints)
fn get_over_time_data(ftl: &mut FtlConnection) -> Result<Vec<TimeStep>, Error> {
    // Read in the length of the data to optimize memory usage
    let map_len = ftl.read_map_len()? as usize;

    // Create the data
    let mut over_time = Vec::with_capacity(map_len);

    // Read in the data
    for _ in 0..map_len {
        let key = ftl.read_i32()?;
        let value = ftl.read_i32()?;
        over_time.push(TimeStep { timestamp: key, count: value });
    }

    Ok(over_time)
}

#[derive(Serialize)]
struct TimeStep {
    timestamp: i32,
    count: i32
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
                    "domains_over_time": [
                        {
                            "timestamp": 1520126228,
                            "count": 10
                        },
                        {
                            "timestamp": 1520126406,
                            "count": 20
                        }
                    ],
                    "blocked_over_time": [
                        {
                            "timestamp": 1520126228,
                            "count": 5
                        },
                        {
                            "timestamp": 1520126406,
                            "count": 5
                        }
                    ]
                })
            )
            .test();
    }
}
