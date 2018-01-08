use util;
use ftl;

use std::collections::HashMap;
use rmp::decode::{ValueReadError, DecodeStringError};
use rmp::Marker;

#[derive(Serialize)]
struct Query(i32, String, String, String, u8, u8);

#[get("/stats/summary")]
pub fn summary() -> util::Reply {
    let mut con = ftl_connect!("stats");

    let domains_blocked = con.read_i32().unwrap();
    let total_queries = con.read_i32().unwrap();
    let blocked_queries = con.read_i32().unwrap();
    let percent_blocked = con.read_f32().unwrap();
    let unique_domains = con.read_i32().unwrap();
    let forwarded_queries = con.read_i32().unwrap();
    let cached_queries = con.read_i32().unwrap();
    let total_clients = con.read_i32().unwrap();
    let unique_clients = con.read_i32().unwrap();
    let status = con.read_u8().unwrap();
    con.expect_eom().unwrap();

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

fn get_top_domains(blocked: bool) -> util::Reply {
    let command = if blocked { "top-ads" } else { "top-domains" };

    let mut con = ftl_connect!(command);
    let queries = con.read_i32().unwrap();

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut top: HashMap<String, i32> = HashMap::new();

    loop {
        let domain = match con.read_str(&mut str_buffer) {
            Ok(domain) => domain,
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

        let count = con.read_i32().unwrap();

        top.insert(domain.to_string(), count);
    }

    let (top_type, queries_type) = if blocked {
        ("top_ads", "blocked_queries")
    } else {
        ("top_domains", "total_queries")
    };

    util::reply_data(json!({
        top_type: top,
        queries_type: queries
    }))
}

#[get("/stats/top_domains")]
pub fn top_domains() -> util::Reply {
    get_top_domains(false)
}

#[get("/stats/top_blocked")]
pub fn top_blocked() -> util::Reply {
    get_top_domains(true)
}

#[get("/stats/top_clients")]
pub fn top_clients() -> util::Reply {
    let mut con = ftl_connect!("top-clients");
    let total_queries = con.read_i32().unwrap();

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut top_clients: HashMap<String, i32> = HashMap::new();

    loop {
        let name = match con.read_str(&mut str_buffer) {
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

        let ip = con.read_str(&mut str_buffer).unwrap();
        let count = con.read_i32().unwrap();

        let key = if ip.len() > 0 {
            format!("{}|{}", name, ip)
        } else {
            name
        };

        top_clients.insert(key, count);
    }

    util::reply_data(json!({
        "top_clients": top_clients,
        "total_queries": total_queries
    }))
}

#[get("/stats/forward_destinations")]
pub fn forward_destinations() -> util::Reply {
    let mut con = ftl_connect!("forward-dest");

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut forward_destinations: HashMap<String, f32> = HashMap::new();

    loop {
        let name = match con.read_str(&mut str_buffer) {
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

        let ip = con.read_str(&mut str_buffer).unwrap();
        let percentage = con.read_f32().unwrap();

        let key = if ip.len() > 0 {
            format!("{}|{}", name, ip)
        } else {
            name
        };

        forward_destinations.insert(key, percentage);
    }

    util::reply_data(json!({
        "forward_destinations": forward_destinations
    }))
}

#[get("/stats/query_types")]
pub fn query_types() -> util::Reply {
    let mut con = ftl_connect!("querytypes");

    let ipv4 = con.read_f32().unwrap();
    let ipv6 = con.read_f32().unwrap();

    util::reply_data(json!({
        "A": ipv4,
        "AAAA": ipv6
    }))
}

#[get("/stats/history")]
pub fn history() -> util::Reply {
    let mut con = ftl_connect!("getallqueries");

    let mut history: Vec<Query> = Vec::new();

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    loop {
        let timestamp = match con.read_i32() {
            Ok(timestamp) => timestamp,
            Err(e) => {
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

        let query_type = con.read_str(&mut str_buffer).unwrap().to_owned();
        let domain = con.read_str(&mut str_buffer).unwrap().to_owned();
        let client = con.read_str(&mut str_buffer).unwrap().to_owned();
        let status = con.read_u8().unwrap();
        let dnssec = con.read_u8().unwrap();

        history.push(Query(timestamp, query_type, domain, client, status, dnssec));
    }

    util::reply_data(json!({
        "history": history
    }))
}

#[get("/stats/recent_blocked")]
pub fn recent_blocked() -> util::Reply {
    let mut con = ftl_connect!("recentBlocked");

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut domains = Vec::new();

    loop {
        let domain = match con.read_str(&mut str_buffer) {
            Ok(domain) => domain.to_owned(),
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

    util::reply_data(json!({
        "recent_blocked": domains
    }))
}

#[get("/stats/overTime/history")]
pub fn over_time_history() -> util::Reply {
    let mut con = ftl_connect!("overTime");

    let domains_over_time = con.read_int_map().unwrap();
    let blocked_over_time = con.read_int_map().unwrap();

    util::reply_data(json!({
        "domains_over_time": domains_over_time,
        "blocked_over_time": blocked_over_time
    }))
}

#[get("/stats/overTime/forward_destinations")]
pub fn over_time_forward_destinations() -> util::Reply {
    let mut con = ftl_connect!("ForwardedoverTime");

    let forward_dest_num = con.read_i32().unwrap();
    let mut over_time: HashMap<i32, Vec<f32>> = HashMap::new();

    loop {
        let timestamp = match con.read_i32() {
            Ok(timestamp) => timestamp,
            Err(e) => {
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

        let mut step = Vec::new();

        for _ in 0..forward_dest_num {
            step.push(con.read_f32().unwrap());
        }

        over_time.insert(timestamp, step);
    }

    util::reply_data(json!({
        "forward_destinations_over_time": over_time
    }))
}
