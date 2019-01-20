// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Functions For Settings Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use std::process::{Command, Stdio};

/// Restart the DNS server (via `pihole restartdns`)
pub fn restart_dns(env: &Env) -> Result<(), Error> {
    // Don't actually run anything during a test
    if env.is_test() {
        return Ok(());
    }

    let status = Command::new("sudo")
        .arg("pihole")
        .arg("restartdns")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context(ErrorKind::RestartDnsError)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::from(ErrorKind::RestartDnsError))
    }
}
