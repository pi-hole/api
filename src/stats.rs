/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Statistic API Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::collections::HashMap;
use rmp::decode::{ValueReadError, DecodeStringError};
use rmp::Marker;
use rocket::State;

use util;
use ftl::FtlConnectionType;

/// Represents a query returned in `/stats/history`
#[derive(Serialize)]
struct Query(i32, String, String, String, u8, u8);

/// Represents a query returned in `/stats/unknown_queries`
#[derive(Serialize)]
struct UnknownQuery(i32, i32, String, String, String, u8, bool);

/// Represents the possible GET parameters on `/stats/top_domains` and `/stats/top_blocked`
#[derive(FromForm)]
pub struct TopParams {
    limit: Option<usize>,
    audit: Option<bool>,
    ascending: Option<bool>
}

impl Default for TopParams {
    /// The default parameters of top_domains and top_blocked requests
    fn default() -> Self {
        TopParams {
            limit: Some(10),
            audit: Some(false),
            ascending: Some(false)
        }
    }
}

/// Represents the possible GET parameters on `/stats/top_clients`
#[derive(FromForm)]
pub struct TopClientParams {
    limit: Option<usize>,
    inactive: Option<bool>,
    ascending: Option<bool>
}

impl Default for TopClientParams {
    /// The default parameters of top_clients requests
    fn default() -> Self {
        TopClientParams {
            limit: Some(10),
            inactive: Some(false),
            ascending: Some(false)
        }
    }
}

/// Represents the possible GET parameters on `/stats/history`
#[derive(FromForm)]
pub struct Timespan {
    from: u64,
    to: u64
}

/// Represents the possible GET parameters on `/stats/recent_blocked`
#[derive(FromForm)]
pub struct RecentBlockedParams {
    num: usize
}

/// Get the summary data
#[get("/stats/summary")]
pub fn summary(ftl: State<FtlConnectionType>) -> util::Reply {
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

/// Get the top domains (blocked or not)
fn get_top_domains(ftl: &FtlConnectionType, blocked: bool, params: TopParams) -> util::Reply {
    let default_limit: usize = TopParams::default().limit.unwrap_or(10);

    // Create the command to send to FTL
    let command = format!(
        "{} ({}) {} {}",
        if blocked { "top-ads" } else { "top-domains" },
        params.limit.unwrap_or(default_limit),
        if params.audit.unwrap_or(false) { "for audit" } else { "" },
        if params.ascending.unwrap_or(false) { "asc" } else { "" }
    );

    // Connect to FTL
    let mut con = ftl.connect(&command)?;

    // Read the number of queries (blocked or total)
    let queries = con.read_i32()?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    // Store the domain -> number data here
    let mut top: HashMap<String, i32> = HashMap::new();

    // Read in the data
    loop {
        // Get the domain, unless we are at the end of the list
        let domain = match con.read_str(&mut str_buffer) {
            Ok(domain) => domain,
            Err(e) => {
                // Check if we received the EOM
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

    // Get the keys to send the data under
    let (top_type, queries_type) = if blocked {
        ("top_blocked", "blocked_queries")
    } else {
        ("top_domains", "total_queries")
    };

    util::reply_data(json!({
        top_type: top,
        queries_type: queries
    }))
}

/// Return the top domains with default parameters
#[get("/stats/top_domains")]
pub fn top_domains(ftl: State<FtlConnectionType>) -> util::Reply {
    get_top_domains(&ftl,false, TopParams::default())
}

/// Return the top domains with specified parameters
#[get("/stats/top_domains?<params>")]
pub fn top_domains_params(ftl: State<FtlConnectionType>, params: TopParams) -> util::Reply {
    get_top_domains(&ftl, false, params)
}

/// Return the top blocked domains with default parameters
#[get("/stats/top_blocked")]
pub fn top_blocked(ftl: State<FtlConnectionType>) -> util::Reply {
    get_top_domains(&ftl, true, TopParams::default())
}

/// Return the top blocked domains with specified parameters
#[get("/stats/top_blocked?<params>")]
pub fn top_blocked_params(ftl: State<FtlConnectionType>, params: TopParams) -> util::Reply {
    get_top_domains(&ftl, true, params)
}

/// Read in the top clients, similar to top_domains and top_blocked but different
fn get_top_clients(ftl: &FtlConnectionType, params: TopClientParams) -> util::Reply {
    let default_limit: usize = 10;

    // Create the command to send to FTL
    let command = format!(
        "top-clients ({}) {} {}",
        params.limit.unwrap_or(default_limit),
        if params.inactive.unwrap_or(false) { "withzero" } else { "" },
        if params.ascending.unwrap_or(false) { "asc" } else { "" }
    );

    // Connect to FTL
    let mut con = ftl.connect(&command)?;

    // Get the total number of queries
    let total_queries = con.read_i32()?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    // Store the hostname -> number data here
    let mut top_clients: HashMap<String, i32> = HashMap::new();

    loop {
        // Get the hostname, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_string(),
            Err(e) => {
                // Check if we received the EOM
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

        // The key will be `hostname|IP` if the hostname exists, otherwise just the IP address
        let key = if name.is_empty() {
            ip.to_owned()
        } else {
            format!("{}|{}", name, ip)
        };

        top_clients.insert(key, count);
    }

    util::reply_data(json!({
        "top_clients": top_clients,
        "total_queries": total_queries
    }))
}

/// Get the top clients with default parameters
#[get("/stats/top_clients")]
pub fn top_clients(ftl: State<FtlConnectionType>) -> util::Reply {
    get_top_clients(&ftl, TopClientParams::default())
}

/// Get the top clients with specified parameters
#[get("/stats/top_clients?<params>")]
pub fn top_clients_params(ftl: State<FtlConnectionType>, params: TopClientParams) -> util::Reply {
    get_top_clients(&ftl, params)
}

/// Get the forward destinations
#[get("/stats/forward_destinations")]
pub fn forward_destinations(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("forward-dest")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut forward_destinations: HashMap<String, f32> = HashMap::new();

    loop {
        // Read in the hostname, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_string(),
            Err(e) => {
                // Check if we received the EOM
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

        // The key will be `hostname|IP` if the hostname exists, otherwise just the IP address
        let key = if ip.len() > 0 {
            format!("{}|{}", name, ip)
        } else {
            name
        };

        forward_destinations.insert(key, percentage);
    }

    util::reply_data(forward_destinations)
}

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

/// Get `num`-many most recently blocked domains
pub fn get_recent_blocked(ftl: &FtlConnectionType, num: usize) -> util::Reply {
    let mut con = ftl.connect(&format!("recentBlocked ({})", num))?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut domains = Vec::with_capacity(num);
    let mut less_domains_than_expected = false;

    for _ in 0..num {
        // Get the next domain. If FTL returns less than what we asked (there haven't been enough
        // blocked domains), then exit the loop
        let domain = match con.read_str(&mut str_buffer) {
            Ok(domain) => domain.to_owned(),
            Err(e) => {
                // Check if we received the EOM
                if let DecodeStringError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        less_domains_than_expected = true;
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        domains.push(domain);
    }

    // If we got the number of domains we expected, then we still need to read the EOM
    if !less_domains_than_expected {
        con.expect_eom()?;
    }

    util::reply_data(domains)
}

/// Get the most recent blocked domain
#[get("/stats/recent_blocked")]
pub fn recent_blocked(ftl: State<FtlConnectionType>) -> util::Reply {
    get_recent_blocked(&ftl, 1)
}

/// Get the `num` most recently blocked domains
#[get("/stats/recent_blocked?<params>")]
pub fn recent_blocked_multi(ftl: State<FtlConnectionType>, params: RecentBlockedParams) -> util::Reply {
    get_recent_blocked(&ftl, params.num)
}

/// Get the names of clients
// TODO: return only the names and IP addresses
#[get("/stats/clients")]
pub fn clients(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("client-names")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut client_data: Vec<(String, String, i32)> = Vec::new();

    loop {
        // Get the hostname, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_owned(),
            Err(e) => {
                // Check if we received the EOM
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

/// Get all unknown queries
#[get("/stats/unknown_queries")]
pub fn unknown_queries(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("unknown")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut queries: Vec<UnknownQuery> = Vec::new();

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

        // Read the rest of the data
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

/// Get the forward destination usage over time
#[get("/stats/overTime/forward_destinations")]
pub fn over_time_forward_destinations(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("ForwardedoverTime")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    // Read in the number of forward destinations
    let forward_dest_num = con.read_i32()? as usize;

    // Create the data structures to store the data in
    let mut forward_data: Vec<String> = Vec::with_capacity(forward_dest_num);
    let mut over_time: HashMap<i32, Vec<f32>> = HashMap::new();

    // Read in forward destination names and IPs
    for _ in 0..forward_dest_num {
        let name = con.read_str(&mut str_buffer)?.to_owned();
        let ip = con.read_str(&mut str_buffer)?.to_owned();

        forward_data.push(
            // The key will be `hostname|IP` if the hostname exists, otherwise just the IP address
            if name.is_empty() {
                ip
            } else {
                format!("{}|{}", name, ip)
            }
        );
    }

    // Get the over time data
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

        // Create a new step in the graph (stores the value of each destination usage at that time)
        let mut step = Vec::with_capacity(forward_dest_num);

        // Read in the forward destination usage
        for _ in 0..forward_dest_num {
            step.push(con.read_f32()?);
        }

        over_time.insert(timestamp, step);
    }

    util::reply_data(json!({
        "forward_destinations": forward_data,
        "over_time": over_time
    }))
}

/// Get the query types usage over time
#[get("/stats/overTime/query_types")]
pub fn over_time_query_types(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("QueryTypesoverTime")?;

    let mut over_time: HashMap<i32, (f32, f32)> = HashMap::new();

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

        let ipv4 = con.read_f32()?;
        let ipv6 = con.read_f32()?;

        over_time.insert(timestamp, (ipv4, ipv6));
    }

    util::reply_data(over_time)
}

/// Get the client queries over time
#[get("/stats/overTime/clients")]
pub fn over_time_clients(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("ClientsoverTime")?;

    let mut over_time: HashMap<i32, Vec<i32>> = HashMap::new();

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

        // Create a new step in the graph (stores the value of each client usage at that time)
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
