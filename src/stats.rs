use util;
use ftl;

use std::collections::HashMap;
use rmp::decode::{ValueReadError, DecodeStringError};
use rmp::Marker;

#[derive(Serialize)]
struct Query(i32, String, String, String, u8, u8);

#[derive(Serialize)]
struct UnknownQuery(i32, i32, String, String, String, u8, bool);

#[derive(FromForm)]
pub struct TopParams {
    limit: Option<usize>,
    audit: Option<bool>,
    desc: Option<bool>
}

impl Default for TopParams {
    fn default() -> Self {
        TopParams {
            limit: Some(10),
            audit: Some(false),
            desc: Some(true)
        }
    }
}

#[get("/stats/summary")]
pub fn summary() -> util::Reply {
    let mut con = ftl::connect("stats")?;

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

fn get_top_domains(blocked: bool, params: TopParams) -> util::Reply {
    let default_limit: usize = 10;

    let command = format!(
        "{} ({}) {} {}",
        if blocked { "top-ads" } else { "top-domains" },
        params.limit.unwrap_or(default_limit),
        if params.audit.unwrap_or(false) { "for audit" } else { "" },
        if params.desc.unwrap_or(true) { "desc" } else { "" }
    );

    let mut con = ftl::connect(&command)?;
    let queries = con.read_i32()?;

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

        let count = con.read_i32()?;

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
    get_top_domains(false, TopParams::default())
}

#[get("/stats/top_domains?<params>")]
pub fn top_domains_params(params: TopParams) -> util::Reply {
    get_top_domains(false, params)
}

#[get("/stats/top_blocked")]
pub fn top_blocked() -> util::Reply {
    get_top_domains(true, TopParams::default())
}

#[get("/stats/top_blocked?<params>")]
pub fn top_blocked_params(params: TopParams) -> util::Reply {
    get_top_domains(true, params)
}

#[get("/stats/top_clients")]
pub fn top_clients() -> util::Reply {
    let mut con = ftl::connect("top-clients")?;
    let total_queries = con.read_i32()?;

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

        let ip = con.read_str(&mut str_buffer)?;
        let count = con.read_i32()?;

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
    let mut con = ftl::connect("forward-dest")?;

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

        let ip = con.read_str(&mut str_buffer)?;
        let percentage = con.read_f32()?;

        let key = if ip.len() > 0 {
            format!("{}|{}", name, ip)
        } else {
            name
        };

        forward_destinations.insert(key, percentage);
    }

    util::reply_data(forward_destinations)
}

#[get("/stats/query_types")]
pub fn query_types() -> util::Reply {
    let mut con = ftl::connect("querytypes")?;

    let ipv4 = con.read_f32()?;
    let ipv6 = con.read_f32()?;
    con.expect_eom()?;

    util::reply_data(json!({
        "A": ipv4,
        "AAAA": ipv6
    }))
}

#[get("/stats/history")]
pub fn history() -> util::Reply {
    let mut con = ftl::connect("getallqueries")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut history: Vec<Query> = Vec::new();

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

        let query_type = con.read_str(&mut str_buffer)?.to_owned();
        let domain = con.read_str(&mut str_buffer)?.to_owned();
        let client = con.read_str(&mut str_buffer)?.to_owned();
        let status = con.read_u8()?;
        let dnssec = con.read_u8()?;

        history.push(Query(timestamp, query_type, domain, client, status, dnssec));
    }

    util::reply_data(history)
}

#[get("/stats/recent_blocked")]
pub fn recent_blocked() -> util::Reply {
    let mut con = ftl::connect("recentBlocked")?;

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

    util::reply_data(domains)
}

#[get("/stats/clients")]
pub fn clients() -> util::Reply {
    let mut con = ftl::connect("client-names")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut client_data: Vec<(String, String, i32)> = Vec::new();

    loop {
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_owned(),
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

        let ip = con.read_str(&mut str_buffer)?.to_owned();
        let count = con.read_i32()?;

        client_data.push((name, ip, count));
    }

    util::reply_data(client_data)
}

#[get("/stats/unknown_queries")]
pub fn unknown_queries() -> util::Reply {
    let mut con = ftl::connect("unknown")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut queries: Vec<UnknownQuery> = Vec::new();

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

        let id = con.read_i32()?;
        let query_type = con.read_str(&mut str_buffer)?.to_owned();
        let domain = con.read_str(&mut str_buffer)?.to_owned();
        let client = con.read_str(&mut str_buffer)?.to_owned();
        let status = con.read_u8()?;
        let complete = con.read_bool()?;

        queries.push(UnknownQuery(timestamp, id, query_type, domain, client, status, complete));
    }

    util::reply_data(queries)
}

#[get("/stats/overTime/history")]
pub fn over_time_history() -> util::Reply {
    let mut con = ftl::connect("overTime")?;

    let domains_over_time = con.read_int_map()?;
    let blocked_over_time = con.read_int_map()?;

    util::reply_data(json!({
        "domains_over_time": domains_over_time,
        "blocked_over_time": blocked_over_time
    }))
}

#[get("/stats/overTime/forward_destinations")]
pub fn over_time_forward_destinations() -> util::Reply {
    let mut con = ftl::connect("ForwardedoverTime")?;

    let forward_dest_num = con.read_i32()?;
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
            step.push(con.read_f32()?);
        }

        over_time.insert(timestamp, step);
    }

    util::reply_data(over_time)
}

#[get("/stats/overTime/query_types")]
pub fn over_time_query_types() -> util::Reply {
    let mut con = ftl::connect("QueryTypesoverTime")?;

    let mut over_time: HashMap<i32, (f32, f32)> = HashMap::new();

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

        let ipv4 = con.read_f32()?;
        let ipv6 = con.read_f32()?;

        over_time.insert(timestamp, (ipv4, ipv6));
    }

    util::reply_data(over_time)
}

#[get("/stats/overTime/clients")]
pub fn over_time_clients() -> util::Reply {
    let mut con = ftl::connect("ClientsoverTime")?;

    let mut over_time: HashMap<i32, Vec<i32>> = HashMap::new();

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

        // Get all the data for this step
        loop {
            let client = con.read_i32()?;

            // Marker for the end of this step
            if client == -1 {
                break;
            }

            step.push(client);
        }

        over_time.insert(timestamp, step);
    }

    util::reply_data(over_time)
}
