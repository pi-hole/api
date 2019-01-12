// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// History Query To JSON Functionality
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{FtlMemory, FtlQuery, ShmLockGuard},
    util::Error
};
use rocket_contrib::json::JsonValue;

/// Create a function to map `FtlQuery` structs to JSON `Value` structs.
pub fn map_query_to_json<'a>(
    ftl_memory: &'a FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<impl Fn(&FtlQuery) -> JsonValue + 'a, Error> {
    let domains = ftl_memory.domains(ftl_lock)?;
    let clients = ftl_memory.clients(ftl_lock)?;
    let strings = ftl_memory.strings(ftl_lock)?;

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

#[cfg(test)]
mod test {
    use super::map_query_to_json;
    use crate::{
        ftl::ShmLockGuard,
        routes::stats::history::testing::{test_memory, test_queries}
    };

    /// Verify that queries are mapped to JSON correctly
    #[test]
    fn test_map_query_to_json() {
        let query = test_queries()[0];
        let ftl_memory = test_memory();
        let map_function = map_query_to_json(&ftl_memory, &ShmLockGuard::Test).unwrap();
        let mapped_query = map_function(&query);

        assert_eq!(
            mapped_query,
            json!({
                "timestamp": 263581,
                "type": 1,
                "status": 2,
                "domain": "domain1.com",
                "client": "client1",
                "dnssec": 1,
                "reply": 3,
                "response_time": 1
            })
        );
    }
}
