// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Functions For Settings Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::Env;
use failure::ResultExt;
use std::process::{Command, Stdio};
use util::{Error, ErrorKind};

/// Restart the DNS server (via `pihole restartdns`)
pub fn restart_dns(env: &Env) -> Result<(), Error> {
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
        Err(ErrorKind::RestartDnsError.into())
    }
}
