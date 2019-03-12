// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Types Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{FtlMemory, FtlQueryType},
    routes::auth::User,
    util::{reply_result, Error, Reply}
};
use rocket::State;

/// Get the query types
#[get("/stats/query_types")]
pub fn query_types(_auth: User, ftl_memory: State<FtlMemory>) -> Reply {
    reply_result(query_types_impl(&ftl_memory))
}

/// Get the query types
fn query_types_impl(ftl_memory: &FtlMemory) -> Result<Vec<QueryTypeReply>, Error> {
    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;

    Ok(FtlQueryType::variants()
        .iter()
        .map(|&variant| QueryTypeReply {
            name: variant.get_name(),
            count: counters.query_type(variant)
        })
        .collect())
}

/// Represents the reply structure for returning query type data
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct QueryTypeReply {
    pub name: String,
    pub count: usize
}

#[cfg(test)]
mod test {
    use super::query_types_impl;
    use crate::{
        ftl::{FtlCounters, FtlMemory, FtlSettings},
        routes::stats::query_types::QueryTypeReply
    };
    use std::collections::HashMap;

    fn test_data() -> FtlMemory {
        FtlMemory::Test {
            counters: FtlCounters {
                query_type_counters: [2, 2, 1, 1, 1, 2, 1],
                total_queries: 10,
                ..FtlCounters::default()
            },
            domains: Vec::new(),
            over_time: Vec::new(),
            strings: HashMap::new(),
            upstreams: Vec::new(),
            queries: Vec::new(),
            clients: Vec::new(),
            settings: FtlSettings::default()
        }
    }

    /// Simple test to validate output
    #[test]
    fn query_types() {
        let expected = vec![
            QueryTypeReply {
                name: "A".to_owned(),
                count: 2
            },
            QueryTypeReply {
                name: "AAAA".to_owned(),
                count: 2
            },
            QueryTypeReply {
                name: "ANY".to_owned(),
                count: 1
            },
            QueryTypeReply {
                name: "SRV".to_owned(),
                count: 1
            },
            QueryTypeReply {
                name: "SOA".to_owned(),
                count: 1
            },
            QueryTypeReply {
                name: "PTR".to_owned(),
                count: 2
            },
            QueryTypeReply {
                name: "TXT".to_owned(),
                count: 1
            },
        ];

        let actual = query_types_impl(&test_data()).unwrap();

        assert_eq!(actual, expected);
    }
}
