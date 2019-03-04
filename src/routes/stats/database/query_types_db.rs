// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Types Endpoint - DB Version
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    ftl::FtlQueryType,
    routes::{auth::User, stats::query_types::QueryTypeReply},
    util::{reply_data, Error, ErrorKind, Reply}
};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, sqlite::SqliteConnection};
use failure::ResultExt;
use std::collections::HashMap;

/// Get query type counts from the database
#[get("/stats/database/query_types?<from>&<until>")]
pub fn query_types_db(from: u64, until: u64, _auth: User, db: FtlDatabase) -> Reply {
    reply_data(query_types_db_impl(from, until, &db as &SqliteConnection)?)
}

/// Get query type counts from the database
fn query_types_db_impl(
    from: u64,
    until: u64,
    db: &SqliteConnection
) -> Result<Vec<QueryTypeReply>, Error> {
    let query_types = get_query_type_counts(db, from, until)?;

    Ok(FtlQueryType::variants()
        .iter()
        .map(|variant| QueryTypeReply {
            name: variant.get_name(),
            count: query_types[variant]
        })
        .collect())
}

/// Get the number of queries with each query type in the specified time range
pub fn get_query_type_counts(
    db: &SqliteConnection,
    from: u64,
    until: u64
) -> Result<HashMap<FtlQueryType, usize>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    let mut counts: HashMap<FtlQueryType, usize> = queries
        // Select the query types and their counts.
        // The raw SQL is used due to a limitation of Diesel, in that it doesn't
        // have full support for mixing aggregate and non-aggregate data when
        // using group_by. See https://github.com/diesel-rs/diesel/issues/1781
        .select((query_type, sql::<BigInt>("COUNT(*)")))
        // Search in the specified time interval
        .filter(timestamp.le(until as i32).and(timestamp.ge(from as i32)))
        // Group the results by query type
        .group_by(query_type)
        // Execute the query
        .get_results::<(i32, i64)>(db)
        // Add error context and check for errors
        .context(ErrorKind::FtlDatabase)?
        // Turn the resulting Vec into an iterator
        .into_iter()
        // Map the values into (FtlQueryType, usize)
        .map(|(q_type, count)| {
            (FtlQueryType::from_number(q_type as isize).unwrap(), count as usize)
        })
        // Turn the iterator into a HashMap
        .collect();

    // Fill in the rest of the query types not found in the database
    for q_type in FtlQueryType::variants() {
        if !counts.contains_key(q_type) {
            counts.insert(*q_type, 0);
        }
    }

    Ok(counts)
}

#[cfg(test)]
mod test {
    use super::get_query_type_counts;
    use crate::{databases::ftl::connect_to_test_db, ftl::FtlQueryType};
    use std::collections::HashMap;

    const FROM_TIMESTAMP: u64 = 0;
    const UNTIL_TIMESTAMP: u64 = 177_180;

    /// Verify the query type counts are accurate
    #[test]
    fn query_type_counts() {
        let mut expected = HashMap::new();
        expected.insert(FtlQueryType::A, 36);
        expected.insert(FtlQueryType::AAAA, 35);
        expected.insert(FtlQueryType::ANY, 0);
        expected.insert(FtlQueryType::SRV, 0);
        expected.insert(FtlQueryType::SOA, 0);
        expected.insert(FtlQueryType::PTR, 23);
        expected.insert(FtlQueryType::TXT, 0);

        let db = connect_to_test_db();
        let actual = get_query_type_counts(&db, FROM_TIMESTAMP, UNTIL_TIMESTAMP).unwrap();

        assert_eq!(actual, expected);
    }
}
