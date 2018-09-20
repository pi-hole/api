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
    cursor: Option<isize>,
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
        // `None::<()>` represents `null` in JSON. It needs the type parameter because
        // it doesn't know what type of Option it is (`Option<T>`)
        return reply_data(json!({
            "cursor": None::<()>,
            "history": []
        }));
    }

    let counters = ftl_memory.counters()?;
    let queries = ftl_memory.queries()?;

    // The following code uses a boxed iterator, Box<Iterator<Item = &FtlQuery>>
    //
    // When you make an iterator chain, it modifies the type of the iterator.
    // Ex. slice.iter().filter(..).map(..) might look like Map<Filter<Iter<T>>, I>
    //
    // Because of this, if you want to dynamically create an iterator like we do
    // below, the iterator must be kept on the heap instead of the stack
    // because the type of the iterator is not known at compile time.
    //
    // What we do know for certain about the iterator is that it implements
    // Iterator<Item = &FtlQuery>, so using Box we can dynamically add many
    // combinations of modifiers to the iterator and not worry about the real
    // type.

    // Start making an iterator by getting valid query references (FTL allocates
    // more than it uses).
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

    // Get the limit
    let limit = params.limit.unwrap_or(100);

    // Apply the limit (plus one to get the cursor) and collect the queries
    let history: Vec<&FtlQuery> = queries_iter.take(limit + 1).collect();

    // Get the next cursor from the the "limit+1"-th query, or
    // the query at index "limit". If no such query exists, the cursor will be None
    // (null in JSON).
    let next_cursor = history.get(limit).map(|query| query.id);

    // Map the queries into the output format
    let history: Vec<Value> = history
        .into_iter()
        // Only take up to the limit this time, not including the last query,
        // because it was just used to get the cursor
        .take(limit)
        .map(map_query_to_json(ftl_memory)?)
        .collect();

    reply_data(json!({
        "cursor": next_cursor,
        "history": history
    }))
}

/// Create a function to map `FtlQuery` structs to JSON `Value` structs.
fn map_query_to_json<'a>(
    ftl_memory: &'a FtlMemory
) -> Result<impl Fn(&FtlQuery) -> Value + 'a, Error> {
    let domains = ftl_memory.domains()?;
    let clients = ftl_memory.clients()?;
    let strings = ftl_memory.strings()?;

    Ok(move |query: &FtlQuery| {
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
}

/// Skip iteration until the query which corresponds to the cursor.
fn skip_to_cursor<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(cursor) = params.cursor {
        Box::new(queries_iter.skip_while(move |query| query.id as isize != cursor))
    } else {
        queries_iter
    }
}

/// Filter out private queries
fn filter_private_queries<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>
) -> Box<Iterator<Item = &'a FtlQuery> + 'a> {
    Box::new(queries_iter.filter(|query| !query.is_private))
}

/// Apply the `SetupVarsEntry::ApiQueryLogShow` setting (`permittedonly`,
/// `blockedonly`, etc).
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

/// Filter out queries before the `from` timestamp
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

/// Filter out queries after the `until` timestamp
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

/// Only show queries with the specified query type
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

/// Only show queries from the specified upstream
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
            let counters = ftl_memory.counters()?;
            let strings = ftl_memory.strings()?;
            let upstreams = ftl_memory.upstreams()?;
            let upstream_id = upstreams
                .iter()
                .take(counters.total_upstreams as usize)
                .position(|item| {
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

/// Only show queries of the specified domain
fn filter_domain<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory
) -> Result<Box<Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref domain_filter) = params.domain {
        // Find the domain. If none can be found, return an empty iterator because no
        // query can match the domain requested
        let counters = ftl_memory.counters()?;
        let strings = ftl_memory.strings()?;
        let domains = ftl_memory.domains()?;
        let domain_id = domains
            .iter()
            .take(counters.total_domains as usize)
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

/// Only show queries of the specified client
fn filter_client<'a>(
    queries_iter: Box<Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory
) -> Result<Box<Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref client_filter) = params.client {
        // Find the client. If none can be found, return an empty iterator because no
        // query can match the client requested
        let counters = ftl_memory.counters()?;
        let strings = ftl_memory.strings()?;
        let clients = ftl_memory.clients()?;
        let client_id = clients
            .iter()
            .take(counters.total_clients as usize)
            .position(|client| {
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
    use super::{
        filter_client, filter_domain, filter_private_queries, filter_query_type,
        filter_setup_vars_setting, filter_time_from, filter_time_until, filter_upstream,
        map_query_to_json, skip_to_cursor, HistoryParams
    };
    use env::{Config, Env, PiholeFile};
    use ftl::{
        FtlClient, FtlCounters, FtlDnssecType, FtlDomain, FtlMemory, FtlQuery, FtlQueryReplyType,
        FtlQueryStatus, FtlQueryType, FtlRegexMatch, FtlUpstream
    };
    use rocket_contrib::Value;
    use std::collections::HashMap;
    use testing::{TestBuilder, TestEnvBuilder};

    /// Shorthand for making `FtlQuery` structs
    macro_rules! query {
        (
            $id:expr,
            $qtype:ident,
            $status:ident,
            $domain:expr,
            $client:expr,
            $upstream:expr,
            $timestamp:expr,
            $private:expr
        ) => {
            FtlQuery::new(
                $id,
                $timestamp,
                1,
                1,
                $domain,
                $client,
                $upstream,
                FtlQueryType::$qtype,
                FtlQueryStatus::$status,
                FtlQueryReplyType::IP,
                FtlDnssecType::Unspecified,
                false,
                true,
                $private
            )
        };
    }

    /// Creates an `FtlMemory` struct from the other test data functions
    fn test_memory() -> FtlMemory {
        FtlMemory::Test {
            clients: test_clients(),
            counters: test_counters(),
            domains: test_domains(),
            over_time: Vec::new(),
            over_time_clients: Vec::new(),
            strings: test_strings(),
            queries: test_queries(),
            upstreams: test_upstreams()
        }
    }

    /// 9 queries. Query 9 is private.
    ///
    /// | ID | Type |   Status   | Domain | Client | Upstream | Timestamp |
    /// | -- | ---- | ---------- | ------ | ------ | -------- | --------- |
    /// | 1  | A    | Forward    | 0      | 0      | 0        | 1         |
    /// | 2  | AAAA | Forward    | 0      | 0      | 0        | 2         |
    /// | 3  | PTR  | Forward    | 0      | 0      | 0        | 3         |
    /// | 4  | A    | Gravity    | 1      | 1      | 0        | 3         |
    /// | 5  | AAAA | Cache      | 0      | 1      | 0        | 4         |
    /// | 6  | AAAA | Wildcard   | 2      | 1      | 0        | 5         |
    /// | 7  | A    | Blacklist  | 3      | 2      | 0        | 5         |
    /// | 8  | AAAA | ExternalB. | 4      | 2      | 1        | 6         |
    /// | 9  | A    | Forward    | 5      | 3      | 0        | 7         |
    fn test_queries() -> Vec<FtlQuery> {
        vec![
            query!(1, A, Forward, 0, 0, 0, 1, false),
            query!(2, AAAA, Forward, 0, 0, 0, 2, false),
            query!(3, PTR, Forward, 0, 0, 0, 3, false),
            query!(4, A, Gravity, 1, 1, 0, 3, false),
            query!(5, AAAA, Cache, 0, 1, 0, 4, false),
            query!(6, AAAA, Wildcard, 2, 1, 0, 5, false),
            query!(7, A, Blacklist, 3, 2, 0, 5, false),
            query!(8, AAAA, ExternalBlock, 4, 2, 1, 6, false),
            query!(9, A, Forward, 5, 3, 0, 7, true),
        ]
    }

    /// The counters necessary for the history tests.
    fn test_counters() -> FtlCounters {
        FtlCounters {
            total_queries: 9,
            total_upstreams: 2,
            total_domains: 6,
            total_clients: 4,
            ..FtlCounters::default()
        }
    }

    /// 6 domains. See `test_queries` for how they're used.
    fn test_domains() -> Vec<FtlDomain> {
        vec![
            FtlDomain::new(4, 0, 1, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 1, 2, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 1, 3, FtlRegexMatch::Blocked),
            FtlDomain::new(1, 1, 4, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 0, 5, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 0, 13, FtlRegexMatch::NotBlocked),
        ]
    }

    /// 4 clients. See `test_queries` for how they're used.
    fn test_clients() -> Vec<FtlClient> {
        vec![
            FtlClient::new(3, 0, 6, Some(7)),
            FtlClient::new(3, 2, 8, None),
            FtlClient::new(2, 2, 9, None),
            FtlClient::new(1, 0, 10, None),
        ]
    }

    /// 1 upstream. See `test_queries` for how it's used.
    fn test_upstreams() -> Vec<FtlUpstream> {
        vec![
            FtlUpstream::new(3, 0, 11, Some(12)),
            FtlUpstream::new(1, 0, 14, Some(15)),
        ]
    }

    /// Strings used in the test data
    fn test_strings() -> HashMap<usize, String> {
        let mut strings = HashMap::new();
        strings.insert(1, "domain1.com".to_owned());
        strings.insert(2, "domain2.com".to_owned());
        strings.insert(3, "domain3.com".to_owned());
        strings.insert(4, "domain4.com".to_owned());
        strings.insert(5, "domain5.com".to_owned());
        strings.insert(6, "192.168.1.10".to_owned());
        strings.insert(7, "client1".to_owned());
        strings.insert(8, "192.168.1.11".to_owned());
        strings.insert(9, "192.168.1.12".to_owned());
        strings.insert(10, "0.0.0.0".to_owned());
        strings.insert(11, "8.8.8.8".to_owned());
        strings.insert(12, "google-public-dns-a.google.com".to_owned());
        strings.insert(13, "hidden".to_owned());
        strings.insert(14, "8.8.4.4".to_owned());
        strings.insert(15, "google-public-dns-b.google.com".to_owned());

        strings
    }

    /// The default behavior lists the first 100 non-private queries sorted by
    /// most recent
    #[test]
    fn default_params() {
        let ftl_memory = test_memory();
        let mut expected_queries = test_queries();

        // The private query should be ignored
        expected_queries.remove(8);

        let history: Vec<Value> = expected_queries
            .iter()
            .rev()
            .map(map_query_to_json(&ftl_memory).unwrap())
            .collect();

        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .ftl_memory(ftl_memory)
            .expect_json(json!({
                "history": history,
                "cursor": None::<()>
            }))
            .test();
    }

    /// When the limit is specified, only that many queries will be shown
    #[test]
    fn limit() {
        let ftl_memory = test_memory();
        let mut expected_queries = test_queries();

        // The private query should be ignored
        expected_queries.remove(8);

        let history: Vec<Value> = expected_queries
            .iter()
            .rev()
            .take(5)
            .map(map_query_to_json(&ftl_memory).unwrap())
            .collect();

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?limit=5")
            .ftl_memory(ftl_memory)
            .expect_json(json!({
                "history": history,
                "cursor": 3
            }))
            .test();
    }

    /// Maximum privacy shows no queries
    #[test]
    fn privacy_max() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=3")
            .ftl_memory(test_memory())
            .expect_json(json!({
                "history": [],
                "cursor": None::<()>
            }))
            .test();
    }

    /// Verify that queries are mapped to JSON correctly
    #[test]
    fn test_map_query_to_json() {
        let query = test_queries()[0];
        let ftl_memory = test_memory();
        let map_function = map_query_to_json(&ftl_memory).unwrap();
        let mapped_query = map_function(&query);

        assert_eq!(
            mapped_query,
            json!({
                "timestamp": 1,
                "type": 0,
                "status": 2,
                "domain": "domain1.com",
                "client": "client1",
                "dnssec": 0,
                "reply": 4,
                "response_time": 1
            })
        );
    }

    /// Skip queries according to the cursor
    #[test]
    fn test_skip_to_cursor() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(4).collect();
        let filtered_queries: Vec<&FtlQuery> = skip_to_cursor(
            Box::new(queries.iter()),
            &HistoryParams {
                cursor: Some(5),
                ..HistoryParams::default()
            }
        ).collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Private queries should not pass the filter
    #[test]
    fn test_filter_private_queries() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().take(8).collect();
        let filtered_queries: Vec<&FtlQuery> =
            filter_private_queries(Box::new(queries.iter())).collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// No queries should be shown if `API_QUERY_LOG_SHOW` equals `nothing`
    #[test]
    fn test_filter_setup_vars_setting_nothing() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_QUERY_LOG_SHOW=nothing")
                .build()
        );
        let queries = test_queries();
        let filtered_queries: Vec<&FtlQuery> =
            filter_setup_vars_setting(Box::new(queries.iter()), &env)
                .unwrap()
                .collect();

        assert_eq!(filtered_queries, Vec::<&FtlQuery>::new());
    }

    /// Only permitted queries should be shown if `API_QUERY_LOG_SHOW` equals
    /// `permittedonly`
    #[test]
    fn test_filter_setup_vars_setting_permitted() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_QUERY_LOG_SHOW=permittedonly")
                .build()
        );
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = vec![
            &queries[0],
            &queries[1],
            &queries[2],
            &queries[4],
            &queries[8],
        ];
        let filtered_queries: Vec<&FtlQuery> =
            filter_setup_vars_setting(Box::new(queries.iter()), &env)
                .unwrap()
                .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only blocked queries should be shown if `API_QUERY_LOG_SHOW` equals
    /// `blockedonly`
    #[test]
    fn test_filter_setup_vars_setting_blocked() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_QUERY_LOG_SHOW=blockedonly")
                .build()
        );
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> =
            vec![&queries[3], &queries[5], &queries[6], &queries[7]];
        let filtered_queries: Vec<&FtlQuery> =
            filter_setup_vars_setting(Box::new(queries.iter()), &env)
                .unwrap()
                .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Skip queries before the timestamp
    #[test]
    fn test_filter_time_from() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(4).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_time_from(
            Box::new(queries.iter()),
            &HistoryParams {
                from: Some(4),
                ..HistoryParams::default()
            }
        ).collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Skip queries after the timestamp
    #[test]
    fn test_filter_time_until() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().take(5).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_time_until(
            Box::new(queries.iter()),
            &HistoryParams {
                until: Some(4),
                ..HistoryParams::default()
            }
        ).collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries with the specified query type
    #[test]
    fn test_filter_query_type() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[3], &queries[6], &queries[8]];
        let filtered_queries: Vec<&FtlQuery> = filter_query_type(
            Box::new(queries.iter()),
            &HistoryParams {
                query_type: Some(FtlQueryType::A),
                ..HistoryParams::default()
            }
        ).collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries with the specified upstream IP
    #[test]
    fn test_filter_upstream_ip() {
        let queries = test_queries();
        let expected_queries = vec![&queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_upstream(
            Box::new(queries.iter()),
            &HistoryParams {
                upstream: Some("8.8.4.4".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory()
        ).unwrap()
            .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries with the specified upstream name
    #[test]
    fn test_filter_upstream_name() {
        let queries = test_queries();
        let expected_queries = vec![&queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_upstream(
            Box::new(queries.iter()),
            &HistoryParams {
                upstream: Some("google-public-dns-b.google.com".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory()
        ).unwrap()
            .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries of the specified domain
    #[test]
    fn test_filter_domain() {
        let queries = test_queries();
        let expected_queries = vec![&queries[3]];
        let filtered_queries: Vec<&FtlQuery> = filter_domain(
            Box::new(queries.iter()),
            &HistoryParams {
                domain: Some("domain2.com".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory()
        ).unwrap()
            .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries from the specified client IP
    #[test]
    fn test_filter_client_ip() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[1], &queries[2]];
        let filtered_queries: Vec<&FtlQuery> = filter_client(
            Box::new(queries.iter()),
            &HistoryParams {
                client: Some("192.168.1.10".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory()
        ).unwrap()
            .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries from the specified client name
    #[test]
    fn test_filter_client_name() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[1], &queries[2]];
        let filtered_queries: Vec<&FtlQuery> = filter_client(
            Box::new(queries.iter()),
            &HistoryParams {
                client: Some("client1".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory()
        ).unwrap()
            .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
