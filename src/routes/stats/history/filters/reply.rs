// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Reply Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Only show queries of the specified reply type
pub fn filter_reply<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(reply) = params.reply {
        Box::new(queries_iter.filter(move |query| query.reply_type == reply))
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::filter_reply;
    use crate::{
        ftl::{FtlQuery, FtlQueryReplyType},
        routes::stats::history::{endpoints::HistoryParams, testing::test_queries}
    };

    /// Only return queries of the specified reply type
    #[test]
    fn test_filter_reply() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0]];
        let filtered_queries: Vec<&FtlQuery> = filter_reply(
            Box::new(queries.iter()),
            &HistoryParams {
                reply: Some(FtlQueryReplyType::CNAME),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
