// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// SetupVars API_QUERY_LOG_SHOW Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries,
    env::Env,
    ftl::{FtlQuery, BLOCKED_STATUSES},
    settings::{ConfigEntry, SetupVarsEntry},
    util::Error
};
use diesel::{prelude::*, sqlite::Sqlite};
use std::iter;

/// Apply the `SetupVarsEntry::ApiQueryLogShow` setting (`permittedonly`,
/// `blockedonly`, etc).
pub fn filter_setup_vars_setting<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    env: &Env
) -> Result<Box<dyn Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    Ok(match SetupVarsEntry::ApiQueryLogShow.read(env)?.as_str() {
        "permittedonly" => Box::new(queries_iter.filter(|query| !query.is_blocked())),
        "blockedonly" => Box::new(queries_iter.filter(|query| query.is_blocked())),
        "nothing" => Box::new(iter::empty()),
        _ => queries_iter
    })
}

/// Apply the `SetupVarsEntry::ApiQueryLogShow` setting (`permittedonly`,
/// `blockedonly`, etc) to database results.
pub fn filter_setup_vars_setting_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    env: &Env
) -> Result<queries::BoxedQuery<'a, Sqlite>, Error> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    Ok(match SetupVarsEntry::ApiQueryLogShow.read(env)?.as_str() {
        "permittedonly" => db_query.filter(status.ne_all(&BLOCKED_STATUSES)),
        "blockedonly" => db_query.filter(status.eq_any(&BLOCKED_STATUSES)),
        "nothing" => db_query.limit(0),
        _ => db_query
    })
}

#[cfg(test)]
mod test {
    use super::{filter_setup_vars_setting, filter_setup_vars_setting_db};
    use crate::{
        databases::ftl::connect_to_test_db,
        env::{Config, Env, PiholeFile},
        ftl::{FtlQuery, BLOCKED_STATUSES},
        routes::stats::history::{database::execute_query, testing::test_queries},
        testing::TestEnvBuilder
    };
    use diesel::prelude::*;

    /// No queries should be shown if `API_QUERY_LOG_SHOW` equals `nothing`
    #[test]
    fn nothing() {
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

        assert_eq!(filtered_queries.len(), 0);
    }

    /// Only permitted queries should be shown if `API_QUERY_LOG_SHOW` equals
    /// `permittedonly`
    #[test]
    fn permitted() {
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
    fn blocked() {
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

    /// No queries should be shown if `API_QUERY_LOG_SHOW` equals `nothing`.
    /// This is a database filter.
    #[test]
    fn nothing_db() {
        use crate::databases::ftl::queries::dsl::*;

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_QUERY_LOG_SHOW=nothing")
                .build()
        );

        let db_query = filter_setup_vars_setting_db(queries.into_boxed(), &env).unwrap();
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        assert_eq!(filtered_queries.len(), 0);
    }

    /// Only permitted queries should be shown if `API_QUERY_LOG_SHOW` equals
    /// `permittedonly`. This is a database filter.
    #[test]
    fn permitted_db() {
        use crate::databases::ftl::queries::dsl::*;

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_QUERY_LOG_SHOW=permittedonly")
                .build()
        );

        let db_query = filter_setup_vars_setting_db(queries.into_boxed(), &env).unwrap();
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        for query in filtered_queries {
            assert!(!BLOCKED_STATUSES.contains(&(query.status as i32)));
        }
    }

    /// Only blocked queries should be shown if `API_QUERY_LOG_SHOW` equals
    /// `blockedonly`. This is a database filter
    #[test]
    fn blocked_db() {
        use crate::databases::ftl::queries::dsl::*;

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_QUERY_LOG_SHOW=blockedonly")
                .build()
        );

        let db_query = filter_setup_vars_setting_db(queries.into_boxed(), &env).unwrap();
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        for query in filtered_queries {
            assert!(BLOCKED_STATUSES.contains(&(query.status as i32)));
        }
    }
}
