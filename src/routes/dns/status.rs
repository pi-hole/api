// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Blocking Status Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Env, PiholeFile},
    routes::dns::common::reload_dns,
    settings::{ConfigEntry, SetupVarsEntry},
    util::{reply_data, reply_error, reply_success, Error, ErrorKind, Reply}
};
use rocket::State;
use rocket_contrib::json::Json;
use std::time::Duration;
use task_scheduler::Scheduler;

/// Get the DNS blocking status
#[get("/dns/status")]
pub fn status(env: State<Env>) -> Reply {
    let status = if SetupVarsEntry::BlockingEnabled.is_true(&env)? {
        "enabled"
    } else {
        "disabled"
    };

    reply_data(json!({ "status": status }))
}

/// Enable/Disable blocking
#[post("/dns/status", data = "<data>")]
pub fn change_status(
    env: State<Env>,
    scheduler: State<Scheduler>,
    data: Json<ChangeStatus>
) -> Reply {
    match (data.action.as_str(), data.time) {
        ("enable", None) => enable(&env)?,
        ("disable", time) => disable(&env, time, Some(&scheduler))?,
        _ => return reply_error(ErrorKind::BadRequest)
    }

    reply_success()
}

/// Enable blocking
fn enable(env: &Env) -> Result<(), Error> {
    // Can't enable blocking when it's already enabled
    if SetupVarsEntry::BlockingEnabled.is_true(&env)? {
        return Err(Error::from(ErrorKind::BadRequest));
    }

    // Restore the backups if they exist
    if env.file_exists(PiholeFile::GravityBackup) {
        env.rename_file(PiholeFile::GravityBackup, PiholeFile::Gravity)?;
    }

    if env.file_exists(PiholeFile::BlackListBackup) {
        env.rename_file(PiholeFile::BlackListBackup, PiholeFile::BlackList)?;
    }

    // Update the blocking status
    SetupVarsEntry::BlockingEnabled.write("true", env)?;

    reload_dns(env)
}

/// Disable blocking. If the time is `None`, then disable permanently.
/// Otherwise, re-enable after the specified number of seconds.
fn disable(env: &Env, time: Option<usize>, scheduler: Option<&Scheduler>) -> Result<(), Error> {
    // Can't disable blocking when it's already disabled
    if !SetupVarsEntry::BlockingEnabled.is_true(&env)? {
        return Err(Error::from(ErrorKind::BadRequest));
    }

    // Backup files if they exist
    if env.file_exists(PiholeFile::Gravity) {
        env.rename_file(PiholeFile::Gravity, PiholeFile::GravityBackup)?;

        // The file will be created and truncated
        env.write_file(PiholeFile::Gravity, false)?;
    }

    if env.file_exists(PiholeFile::BlackList) {
        env.rename_file(PiholeFile::BlackList, PiholeFile::BlackListBackup)?;

        // The file will be created and truncated
        env.write_file(PiholeFile::BlackList, false)?;
    }

    // Update the blocking status
    SetupVarsEntry::BlockingEnabled.write("false", env)?;

    reload_dns(env)?;

    // Don't schedule the re-enable when testing. The Clone implementation for
    // Env::Test is not available (crashes due to unimplemented!()), and we
    // don't want to be scheduling work which runs after the tests.
    if !env.is_test() {
        // The scheduler should always be given when not in a test
        assert!(
            scheduler.is_some(),
            "Scheduler must be supplied when not testing"
        );

        // Check if we should re-enable after a specified timeout
        if let Some(time) = time {
            // Make a copy of the Env to move to the scheduler thread
            let env_copy = env.clone();

            // Re-enable blocking after the timeout
            scheduler
                .unwrap()
                .after_duration(Duration::from_secs(time as u64), move || {
                    // Handle the result of enabling, so that if it's an error
                    // the thread does not panic
                    if let Err(e) = enable(&env_copy) {
                        if e.kind() == ErrorKind::BadRequest {
                            // If it was a bad request, blocking was probably
                            // already re-enabled. This is a fairly common
                            // scenario, so no error should be logged.
                            return;
                        }

                        e.print_stacktrace();
                    }
                });
        }
    }

    Ok(())
}

/// Represents the API input for changing the DNS blocking status
#[derive(Deserialize)]
pub struct ChangeStatus {
    /// The action to perform. Should be either "enable" or "disable".
    action: String,

    /// The number of seconds to wait before re-enabling. Should be None when
    /// the action is "enable".
    time: Option<usize>
}

#[cfg(test)]
mod test {
    use super::{disable, enable};
    use crate::{
        env::{Config, Env, PiholeFile},
        testing::{TestBuilder, TestEnvBuilder},
        util::ErrorKind
    };
    use rocket::http::Method;

    /// Return enabled status if blocking is enabled
    #[test]
    fn read_enabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=true")
            .expect_json(json!({ "status": "enabled" }))
            .test();
    }

    /// Return disabled status if blocking is disabled
    #[test]
    fn read_disabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=false")
            .expect_json(json!({ "status": "disabled" }))
            .test();
    }

    /// Return enabled status if blocking status is unknown
    #[test]
    fn read_default() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "")
            .expect_json(json!({ "status": "enabled" }))
            .test();
    }

    /// Enable blocking if it's disabled
    #[test]
    fn action_enable() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .method(Method::Post)
            .body(json!({ "action": "enable" }))
            .file_expect(
                PiholeFile::SetupVars,
                "BLOCKING_ENABLED=false\n",
                "BLOCKING_ENABLED=true\n"
            )
            .file_expect(PiholeFile::Gravity, "", "127.0.0.1 localhost")
            .file_expect(PiholeFile::GravityBackup, "127.0.0.1 localhost", "")
            .file_expect(PiholeFile::BlackList, "", "ad.domain")
            .file_expect(PiholeFile::BlackListBackup, "ad.domain", "")
            .expect_json(json!({ "status": "success" }))
            .test();
    }

    /// Return an error if blocking is enabled and we try to enable it again
    #[test]
    fn action_enable_error() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=true")
                .build()
        );

        assert_eq!(
            enable(&env).map_err(|e| e.kind()),
            Err(ErrorKind::BadRequest)
        );
    }

    /// Disable blocking if it's enabled
    #[test]
    fn action_disable() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .method(Method::Post)
            .body(json!({ "action": "disable" }))
            .file_expect(
                PiholeFile::SetupVars,
                "BLOCKING_ENABLED=true\n",
                "BLOCKING_ENABLED=false\n"
            )
            .file_expect(PiholeFile::Gravity, "127.0.0.1 localhost", "")
            .file_expect(PiholeFile::GravityBackup, "", "127.0.0.1 localhost")
            .file_expect(PiholeFile::BlackList, "ad.domain", "")
            .file_expect(PiholeFile::BlackListBackup, "", "ad.domain")
            .expect_json(json!({ "status": "success" }))
            .test();
    }

    /// Return an error if blocking is disabled and we try to disable it again
    #[test]
    fn action_disable_error() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=false")
                .build()
        );

        assert_eq!(
            disable(&env, None, None).map_err(|e| e.kind()),
            Err(ErrorKind::BadRequest)
        );
    }
}
