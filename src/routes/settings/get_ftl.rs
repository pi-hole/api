// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Settings
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use env::Env;
use rocket::State;
use settings::{read_ftl_conf, FtlConfEntry};
use util::{reply_data, Reply};

/// Read FTL's settings
#[get("/settings/ftl")]
pub fn get_ftl(env: State<Env>, _auth: User) -> Reply {
    // if setting is not present, report default
    let socket_listening =
        read_ftl_conf(FtlConfEntry::SocketListening, &env)?.unwrap_or("localonly".to_owned());
    let query_display =
        read_ftl_conf(FtlConfEntry::QueryDisplay, &env)?.unwrap_or("yes".to_owned());
    let aaaa_query_analysis =
        read_ftl_conf(FtlConfEntry::AaaaQueryAnalysis, &env)?.unwrap_or("yes".to_owned());
    let resolve_ipv6 = read_ftl_conf(FtlConfEntry::ResolveIpv6, &env)?.unwrap_or("yes".to_owned());
    let resolve_ipv4 = read_ftl_conf(FtlConfEntry::ResolveIpv4, &env)?.unwrap_or("yes".to_owned());
    let max_db_days: i32 = read_ftl_conf(FtlConfEntry::MaxDbDays, &env)?
        .unwrap_or("365".to_owned())
        .parse()
        .unwrap_or(365);
    let db_interval: f32 = read_ftl_conf(FtlConfEntry::DbInterval, &env)?
        .unwrap_or("1.0".to_owned())
        .parse()
        .unwrap_or(1.0);
    let db_file = read_ftl_conf(FtlConfEntry::DbFile, &env)?.unwrap_or("".to_owned());
    let max_log_age: f32 = read_ftl_conf(FtlConfEntry::MaxLogAge, &env)?
        .unwrap_or("24.0".to_owned())
        .parse()
        .unwrap_or(24.0);
    let ftl_port: i16 = read_ftl_conf(FtlConfEntry::FtlPort, &env)?
        .unwrap_or("4711".to_owned())
        .parse()
        .unwrap_or(4711);
    let privacy_level: i32 = read_ftl_conf(FtlConfEntry::PrivacyLevel, &env)?
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap_or(0);
    let ignore_local_host =
        read_ftl_conf(FtlConfEntry::IgnoreLocalHost, &env)?.unwrap_or("no".to_owned());
    let blocking_mode =
        read_ftl_conf(FtlConfEntry::BlockingMode, &env)?.unwrap_or("NULL".to_owned());

    reply_data(json!({
        "socket_listening": socket_listening,
        "query_display": query_display,
        "aaaa_query_analysis": aaaa_query_analysis,
        "resolve_ipv6": resolve_ipv6,
        "resolve_ipv4": resolve_ipv4,
        "max_db_days": max_db_days,
        "db_interval": db_interval,
        "db_file": db_file,
        "max_log_age": max_log_age,
        "ftl_port": ftl_port,
        "privacy_level": privacy_level,
        "ignore_local_host": ignore_local_host,
        "blocking_mode": blocking_mode
    }))
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use testing::TestBuilder;

    /// Test that default settings are reported if not present
    #[test]
    fn test_get_ftl() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/ftl")
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!({
                "socket_listening": "localonly",
                "query_display": "yes",
                "aaaa_query_analysis": "yes",
                "resolve_ipv6": "yes",
                "resolve_ipv4": "yes",
                "max_db_days": 365,
                "db_interval": 1.0,
                "db_file": "",
                "max_log_age": 24.0,
                "ftl_port": 4711,
                "privacy_level": 0,
                "ignore_local_host": "no",
                "blocking_mode": "NULL"
            }))
            .test();
    }
}
