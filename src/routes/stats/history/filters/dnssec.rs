// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// DNSSEC Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Only show queries of the specified DNSSEC type
pub fn filter_dnssec<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(dnssec) = params.dnssec {
        Box::new(queries_iter.filter(move |query| query.dnssec_type == dnssec))
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::filter_dnssec;
    use crate::{
        ftl::{FtlDnssecType, FtlQuery},
        routes::stats::history::{endpoints::HistoryParams, testing::test_queries}
    };

    /// Only return queries of the specified DNSSEC type
    #[test]
    fn test_filter_dnssec() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0]];
        let filtered_queries: Vec<&FtlQuery> = filter_dnssec(
            Box::new(queries.iter()),
            &HistoryParams {
                dnssec: Some(FtlDnssecType::Secure),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
