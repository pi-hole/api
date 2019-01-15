// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
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
        ("disable", time) => disable(&env, time, &scheduler)?,
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
fn disable(env: &Env, time: Option<usize>, scheduler: &Scheduler) -> Result<(), Error> {
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
        // Check if we should re-enable after a specified timeout
        if let Some(time) = time {
            // Make a copy of the Env to move to the scheduler thread
            let env_copy = env.clone();

            // Re-enable blocking after the timeout
            scheduler.after_duration(Duration::from_secs(time as u64), move || {
                enable(&env_copy).unwrap()
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
    use crate::{env::PiholeFile, testing::TestBuilder};

    #[test]
    fn test_status_enabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=true")
            .expect_json(json!({ "status": "enabled" }))
            .test();
    }

    #[test]
    fn test_status_disabled() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=false")
            .expect_json(json!({ "status": "disabled" }))
            .test();
    }

    #[test]
    fn test_status_default() {
        TestBuilder::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::SetupVars, "")
            .expect_json(json!({ "status": "enabled" }))
            .test();
    }
}
