// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// History Query To JSON Functionality
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{FtlMemory, FtlQuery, ShmLockGuard},
    routes::stats::{
        common::{HIDDEN_CLIENT, HIDDEN_DOMAIN},
        QueryReply
    },
    settings::FtlPrivacyLevel,
    util::Error
};

/// Create a function to map `FtlQuery` structs to JSON `Value` structs. The
/// queries' privacy levels will be taken into account when exposing their data.
pub fn map_query_to_json<'a>(
    ftl_memory: &'a FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<impl Fn(&FtlQuery) -> QueryReply + 'a, Error> {
    let domains = ftl_memory.domains(ftl_lock)?;
    let clients = ftl_memory.clients(ftl_lock)?;
    let strings = ftl_memory.strings(ftl_lock)?;

    Ok(move |query: &FtlQuery| {
        // Get the domain depending on the privacy level
        let domain = if query.privacy_level < FtlPrivacyLevel::HideDomains {
            domains[query.domain_id as usize].get_domain(&strings)
        } else {
            HIDDEN_DOMAIN
        };

        // Get the client depending on the privacy level
        let client = if query.privacy_level < FtlPrivacyLevel::HideDomainsAndClients {
            let client = clients[query.client_id as usize];

            // Try to get the client name first, but if it doesn't exist use the IP
            client
                .get_name(&strings)
                .unwrap_or_else(|| client.get_ip(&strings))
        } else {
            HIDDEN_CLIENT
        };

        // Check if response was received (response time should be smaller than 30min)
        let response_time = if query.response_time < 18_000_000 {
            query.response_time
        } else {
            0
        } as u32;

        QueryReply {
            timestamp: query.timestamp as u64,
            r#type: query.query_type as u8,
            status: query.status as u8,
            domain: domain.to_owned(),
            client: client.to_owned(),
            dnssec: query.dnssec_type as u8,
            reply: query.reply_type as u8,
            response_time
        }
    })
}

#[cfg(test)]
mod test {
    use super::map_query_to_json;
    use crate::{
        ftl::ShmLockGuard,
        routes::stats::{
            history::testing::{test_memory, test_queries},
            QueryReply
        },
        settings::FtlPrivacyLevel
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
            QueryReply {
                timestamp: 263_581,
                r#type: 1,
                status: 2,
                domain: "domain1.com".to_owned(),
                client: "client1".to_owned(),
                dnssec: 1,
                reply: 3,
                response_time: 1
            }
        );
    }

    /// When the query's privacy level hides domains, hide the domain
    #[test]
    fn private_domains() {
        let mut query = test_queries()[0];
        let ftl_memory = test_memory();
        let map_function = map_query_to_json(&ftl_memory, &ShmLockGuard::Test).unwrap();

        query.privacy_level = FtlPrivacyLevel::HideDomains;
        let mapped_query = map_function(&query);

        assert_eq!(
            mapped_query,
            QueryReply {
                timestamp: 263_581,
                r#type: 1,
                status: 2,
                domain: "hidden".to_owned(),
                client: "client1".to_owned(),
                dnssec: 1,
                reply: 3,
                response_time: 1
            }
        );
    }

    /// When the query's privacy level hides clients, hide the client
    #[test]
    fn private_clients() {
        let mut query = test_queries()[0];
        let ftl_memory = test_memory();
        let map_function = map_query_to_json(&ftl_memory, &ShmLockGuard::Test).unwrap();

        query.privacy_level = FtlPrivacyLevel::HideDomainsAndClients;
        let mapped_query = map_function(&query);

        assert_eq!(
            mapped_query,
            QueryReply {
                timestamp: 263_581,
                r#type: 1,
                status: 2,
                domain: "hidden".to_owned(),
                client: "0.0.0.0".to_owned(),
                dnssec: 1,
                reply: 3,
                response_time: 1
            }
        );
    }
}
