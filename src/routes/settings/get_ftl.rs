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
use routes::settings::common::as_bool;
use settings::{ConfigEntry, FtlConfEntry};
use util::{reply_data, Reply};

/// Read FTL's settings
#[get("/settings/ftl")]
pub fn get_ftl(env: State<Env>, _auth: User) -> Reply {
    // if setting is not present, report default
    let socket_listening = FtlConfEntry::SocketListening.read(&env)?;
    let query_display = FtlConfEntry::QueryDisplay.read(&env)?;
    let aaaa_query_analysis = FtlConfEntry::AaaaQueryAnalysis.read(&env)?;
    let resolve_ipv6 = FtlConfEntry::ResolveIpv6.read(&env)?;
    let resolve_ipv4 = FtlConfEntry::ResolveIpv4.read(&env)?;
    let max_db_days: i32 = FtlConfEntry::MaxDbDays
        .read(&env)?
        .parse()
        .unwrap_or_default();
    let db_interval: f32 = FtlConfEntry::DbInterval
        .read(&env)?
        .parse()
        .unwrap_or_default();
    let db_file = FtlConfEntry::DbFile.read(&env)?;
    let max_log_age: f32 = FtlConfEntry::MaxLogAge
        .read(&env)?
        .parse()
        .unwrap_or_default();
    let ftl_port: usize = FtlConfEntry::FtlPort
        .read(&env)?
        .parse()
        .unwrap_or_default();
    let privacy_level: i32 = FtlConfEntry::PrivacyLevel
        .read(&env)?
        .parse()
        .unwrap_or_default();
    let ignore_local_host = FtlConfEntry::IgnoreLocalHost.read(&env)?;
    let blocking_mode = FtlConfEntry::BlockingMode.read(&env)?;
    let regex_debug_mode: bool = FtlConfEntry::RegexDebugMode
        .read(&env)?
        .parse()
        .unwrap_or_default();

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
        "blocking_mode": blocking_mode,
        "regex_debug_mode": regex_debug_mode
    }))
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use testing::TestBuilder;

    /// Test that correct settings are reported from populated file
    #[test]
    fn test_get_ftl_populated() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/ftl")
            .file(
                PiholeFile::FtlConfig,
                "SOCKET_LISTENING=all\n\
                 QUERY_DISPLAY=no\n\
                 AAAA_QUERY_ANALYSIS=no\n\
                 RESOLVE_IPV6=no\n\
                 RESOLVE_IPV4=no\n\
                 MAXDBDAYS=30\n\
                 DBINTERVAL=3.0\n\
                 DBFILE=/etc/pihole/test/pihole-FTL.db\n\
                 MAXLOGAGE=48.0\n\
                 FTLPORT=38911\n\
                 PRIVACYLEVEL=2\n\
                 IGNORE_LOCALHOST=yes\n\
                 BLOCKINGMODE=NXDOMAIN\n\
                 REGEX_DEBUGMODE=true\n"
            )
            .expect_json(json!({
                "socket_listening": "all",
                "query_display": "no",
                "aaaa_query_analysis": "no",
                "resolve_ipv6": "no",
                "resolve_ipv4": "no",
                "max_db_days": 30,
                "db_interval": 3.0,
                "db_file": "/etc/pihole/test/pihole-FTL.db",
                "max_log_age": 48.0,
                "ftl_port": 38911,
                "privacy_level": 2,
                "ignore_local_host": "yes",
                "blocking_mode": "NXDOMAIN",
                "regex_debug_mode": true
            }))
            .test();
    }

    /// Test that default settings are reported if not present
    #[test]
    fn test_get_ftl_default() {
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
                "db_file": "/etc/pihole/pihole-FTL.db",
                "max_log_age": 24.0,
                "ftl_port": 4711,
                "privacy_level": 0,
                "ignore_local_host": "no",
                "blocking_mode": "NULL",
                "regex_debug_mode": false
            }))
            .test();
    }
}
