// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Private Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, settings::FtlPrivacyLevel};

/// Filter out private queries
pub fn filter_private_queries<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    Box::new(queries_iter.filter(|query| query.privacy_level < FtlPrivacyLevel::Maximum))
}

#[cfg(test)]
mod test {
    use super::filter_private_queries;
    use crate::{ftl::FtlQuery, routes::stats::history::testing::test_queries};

    /// Private queries should not pass the filter
    #[test]
    fn test_filter_private_queries() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().take(8).collect();
        let filtered_queries: Vec<&FtlQuery> =
            filter_private_queries(Box::new(queries.iter())).collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
