// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query History Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use env::Env;
use ftl::{FtlMemory, FtlQuery, FtlQueryStatus, FtlQueryType};
use rocket::State;
use rocket_contrib::Value;
use settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel, SetupVarsEntry};
use std::iter;
use util::{reply_data, Error, Reply};

/// Get the entire query history (as stored in FTL)
#[get("/stats/history")]
pub fn history(_auth: User, ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    get_history(&ftl_memory, &env, HistoryParams::default())
}

/// Get the query history according to the specified parameters
#[get("/stats/history?<params>")]
pub fn history_params(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: HistoryParams
) -> Reply {
    get_history(&ftl_memory, &env, params)
}

/// Represents the possible GET parameters on `/stats/history`
#[derive(FromForm)]
pub struct HistoryParams {
    cursor: Option<usize>,
    from: Option<u64>,
    until: Option<u64>,
    domain: Option<String>,
    client: Option<String>,
    upstream: Option<String>,
    query_type: Option<FtlQueryType>,
    limit: Option<usize>
}

impl Default for HistoryParams {
    fn default() -> Self {
        HistoryParams {
            cursor: None,
            from: None,
            until: None,
            domain: None,
            client: None,
            upstream: None,
            query_type: None,
            limit: Some(100)
        }
    }
}

/// Get the query history according to the specified parameters
fn get_history(ftl_memory: &FtlMemory, env: &Env, params: HistoryParams) -> Reply {
    // Check if query details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)? >= FtlPrivacyLevel::Maximum {
        return reply_data([0; 0]);
    }

    let counters = ftl_memory.counters()?;
    let strings = ftl_memory.strings()?;
    let queries = ftl_memory.queries()?;
    let domains = ftl_memory.domains()?;
    let clients = ftl_memory.clients()?;

    // The following code uses a boxed iterator, Box<Iterator<Item = &FtlQuery>>
    //
    // When you make an iterator chain, it modifies the type of the iterator.
    // Ex. slice.iter().filter(..).map(..) might look like Map<Filter<Iter<T>, ..>,
    // ..>
    //
    // Because of this, if you want to dynamically create an iterator like we do
    // below, the iterator must be kept on the heap instead of the stack
    // because the type of the iterator is not known at compile time.
    //
    // What we do know for certain about the iterator is that it implements
    // Iterator<Item = &FtlQuery>, so using Box we can dynamically add many
    // combinations of modifiers to the iterator and not worry about the real
    // type.

    // Start making an iterator by getting valid query references (FTL
    // allocates more than it uses).
    let queries_iter = Box::new(
        queries
            .iter()
            // Get the most recent queries first
            .rev()
            // Skip the uninitialized queries
            .skip(queries.len() - counters.total_queries as usize)
    );

    // If there is a cursor, skip to the referenced query
    let queries_iter = skip_to_cursor(queries_iter, &params);

    // Apply filters
    let queries_iter = filter_private_queries(queries_iter);
    let queries_iter = filter_setup_vars_setting(queries_iter, env)?;
    let queries_iter = filter_time_from(queries_iter, &params);
    let queries_iter = filter_time_until(queries_iter, &params);
    let queries_iter = filter_query_type(queries_iter, &params);
    let queries_iter = filter_upstream(queries_iter, &params, ftl_memory)?;
    let queries_iter = filter_domain(queries_iter, &params, ftl_memory)?;
    let queries_iter = filter_client(queries_iter, &params, ftl_memory)?;

    // Apply the limit
    let queries_iter = queries_iter.take(params.limit.unwrap_or(100));

    // Collect the queries so we can get the next cursor
    let history: Vec<&FtlQuery> = queries_iter.collect();

    // Get the next cursor from the last query returned
    let next_cursor = history.last().map(|query| query.id - 1).unwrap_or_default();

    // Map the queries into the output format
    let history: Vec<Value> = history
        .into_iter()
        .map(|query| {
            let domain = domains[query.domain_id as usize].get_domain(&strings);
            let client = clients[query.client_id as usize];

            // Try to get the client name first, but if it doesn't exist use the IP
            let client = client
                .get_name(&strings)
                .unwrap_or_else(|| client.get_ip(&strings));

            // Check if response was received (response time should be smaller than 30min)
            let response_time = if query.response_time < 18_000_000 {
                query.response_time
            } else {
                0
            };

            json!({
                "timestamp": query.timestamp,
                "type": query.query_type as u8,
                "status": query.status as u8,
                "domain": domain,
                "client": client,
                "dnssec": query.dnssec_type as u8,
                "reply": query.reply_type as u8,
                "response_time": response_time
            })
        })
        .collect();

    reply_data(json!({
        "cursor": next_cursor,
        "history": history
    }))
}

fn skip_to_cursor<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(cursor) = params.cursor {
        Box::new(queries_iter.skip_while(move |query| query.id as usize != cursor))
    } else {
        queries_iter
    }
}

fn filter_private_queries<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    Box::new(queries_iter.filter(|query| !query.is_private))
}

fn filter_setup_vars_setting<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    env: &Env
) -> Result<Box<Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    Ok(match SetupVarsEntry::ApiQueryLogShow.read(env)?.as_str() {
        "permittedonly" => Box::new(queries_iter.filter(|query| match query.status {
            FtlQueryStatus::Forward | FtlQueryStatus::Cache => true,
            _ => false
        })),
        "blockedonly" => Box::new(queries_iter.filter(|query| match query.status {
            FtlQueryStatus::Gravity
            | FtlQueryStatus::Blacklist
            | FtlQueryStatus::Wildcard
            | FtlQueryStatus::ExternalBlock => true,
            _ => false
        })),
        "nothing" => Box::new(iter::empty()),
        _ => queries_iter
    })
}

fn filter_time_from<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(from) = params.from {
        Box::new(queries_iter.filter(move |query| query.timestamp as u64 >= from))
    } else {
        queries_iter
    }
}

fn filter_time_until<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(until) = params.until {
        Box::new(queries_iter.filter(move |query| query.timestamp as u64 <= until))
    } else {
        queries_iter
    }
}

fn filter_query_type<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(query_type) = params.query_type {
        Box::new(queries_iter.filter(move |query| query.query_type == query_type))
    } else {
        queries_iter
    }
}

fn filter_upstream<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory
) -> Result<Box<Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref upstream) = params.upstream {
        if upstream == "blocklist" {
            Ok(Box::new(queries_iter.filter(|query| match query.status {
                FtlQueryStatus::Gravity | FtlQueryStatus::Blacklist | FtlQueryStatus::Wildcard => {
                    true
                }
                _ => false
            })))
        } else if upstream == "cache" {
            Ok(Box::new(
                queries_iter.filter(|query| query.status == FtlQueryStatus::Cache)
            ))
        } else {
            // Find the upstream. If none can be found, return an empty iterator because no
            // query can match the upstream requested
            let strings = ftl_memory.strings()?;
            let upstreams = ftl_memory.upstreams()?;
            let upstream_id = upstreams.iter().position(|item| {
                let ip = item.get_ip(&strings);
                let name = item.get_name(&strings);

                ip == upstream || if let Some(name) = name {
                    name == upstream
                } else {
                    false
                }
            });

            if let Some(upstream_id) = upstream_id {
                Ok(Box::new(queries_iter.filter(move |query| {
                    query.upstream_id as usize == upstream_id
                })))
            } else {
                Ok(Box::new(iter::empty()))
            }
        }
    } else {
        Ok(queries_iter)
    }
}

fn filter_domain<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory
) -> Result<Box<Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref domain_filter) = params.domain {
        // Find the domain. If none can be found, return an empty iterator because no
        // query can match the domain requested
        let strings = ftl_memory.strings()?;
        let domains = ftl_memory.domains()?;
        let domain_id = domains
            .iter()
            .position(|domain| domain.get_domain(&strings) == domain_filter);

        if let Some(domain_id) = domain_id {
            Ok(Box::new(queries_iter.filter(move |query| {
                query.domain_id as usize == domain_id
            })))
        } else {
            Ok(Box::new(iter::empty()))
        }
    } else {
        Ok(queries_iter)
    }
}

fn filter_client<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory
) -> Result<Box<Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref client_filter) = params.client {
        // Find the client. If none can be found, return an empty iterator because no
        // query can match the client requested
        let strings = ftl_memory.strings()?;
        let clients = ftl_memory.clients()?;
        let client_id = clients.iter().position(|client| {
            let ip = client.get_ip(&strings);
            let name = client.get_name(&strings);

            ip == client_filter || if let Some(name) = name {
                name == client_filter
            } else {
                false
            }
        });

        if let Some(client_id) = client_id {
            Ok(Box::new(queries_iter.filter(move |query| {
                query.client_id as usize == client_id
            })))
        } else {
            Ok(Box::new(iter::empty()))
        }
    } else {
        Ok(queries_iter)
    }
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{write_eom, TestBuilder};

    #[test]
    fn test_history() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_str(&mut data, "IPv6").unwrap();
        encode::write_str(&mut data, "doubleclick.com").unwrap();
        encode::write_str(&mut data, "client2").unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .ftl("getallqueries", data)
            .expect_json(json!([
                [1520126228, "IPv4", "example.com", "client1", 2, 1],
                [1520126406, "IPv6", "doubleclick.com", "client2", 1, 1]
            ]))
            .test();
    }

    #[test]
    fn test_history_timespan() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_str(&mut data, "IPv6").unwrap();
        encode::write_str(&mut data, "doubleclick.com").unwrap();
        encode::write_str(&mut data, "client2").unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?from=1520126228&until=1520126406")
            .ftl("getallqueries-time 1520126228 1520126406", data)
            .expect_json(json!([
                [1520126228, "IPv4", "example.com", "client1", 2, 1],
                [1520126406, "IPv6", "doubleclick.com", "client2", 1, 1]
            ]))
            .test();
    }

    #[test]
    fn test_history_domain() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?domain=example.com")
            .ftl("getallqueries-domain example.com", data)
            .expect_json(json!([[
                1520126228,
                "IPv4",
                "example.com",
                "client1",
                2,
                1
            ]]))
            .test();
    }

    #[test]
    fn test_history_client() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?client=client1")
            .ftl("getallqueries-client client1", data)
            .expect_json(json!([[
                1520126228,
                "IPv4",
                "example.com",
                "client1",
                2,
                1
            ]]))
            .test();
    }
}
