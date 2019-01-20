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
    env::Env,
    ftl::FtlQuery,
    settings::{ConfigEntry, SetupVarsEntry},
    util::Error
};
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

#[cfg(test)]
mod test {
    use super::filter_setup_vars_setting;
    use crate::{
        env::{Config, Env, PiholeFile},
        ftl::FtlQuery,
        routes::stats::history::testing::test_queries,
        testing::TestEnvBuilder
    };

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
}
