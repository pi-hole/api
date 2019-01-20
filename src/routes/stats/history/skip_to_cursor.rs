// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// History Skip To Cursor Functionality
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Skip iteration until the query which corresponds to the cursor.
pub fn skip_to_cursor<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(cursor) = params.cursor {
        if let Some(id) = cursor.id {
            Box::new(queries_iter.skip_while(move |query| query.id as i32 != id))
        } else if let Some(db_id) = cursor.db_id {
            Box::new(queries_iter.skip_while(move |query| query.database_id != db_id))
        } else {
            // No cursor data, don't skip any queries
            queries_iter
        }
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::skip_to_cursor;
    use crate::{
        ftl::FtlQuery,
        routes::stats::history::{
            endpoints::{HistoryCursor, HistoryParams},
            testing::test_queries
        }
    };

    /// Skip queries according to the cursor (dnsmasq ID)
    #[test]
    fn test_skip_to_cursor_dnsmasq() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(7).collect();
        let filtered_queries: Vec<&FtlQuery> = skip_to_cursor(
            Box::new(queries.iter()),
            &HistoryParams {
                cursor: Some(HistoryCursor {
                    id: Some(8),
                    db_id: None
                }),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Skip queries according to the cursor (database ID)
    #[test]
    fn test_skip_to_cursor_database() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(4).collect();
        let filtered_queries: Vec<&FtlQuery> = skip_to_cursor(
            Box::new(queries.iter()),
            &HistoryParams {
                cursor: Some(HistoryCursor {
                    id: None,
                    db_id: Some(99)
                }),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
