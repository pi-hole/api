// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Functions For DNS Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid
};
use std::process::Command;

/// Reload the DNS server to activate config changes
pub fn reload_dns(env: &Env) -> Result<(), Error> {
    // Don't actually reload the DNS server during testing
    if env.is_test() {
        return Ok(());
    }

    // Get the PID of FTLDNS. There doesn't seem to be a better way than to run
    // pidof in a shell.
    let output = Command::new("pidof")
        .arg("pihole-FTL")
        .output()
        .context(ErrorKind::ReloadDnsError)?;

    // Check if it returned successfully
    if !output.status.success() {
        return Err(Error::from(ErrorKind::ReloadDnsError));
    }

    // Parse the output for the PID
    let pid_str = String::from_utf8_lossy(&output.stdout);
    let pid = pid_str
        .trim()
        .parse::<usize>()
        .context(ErrorKind::ReloadDnsError)?;

    // Send SIGHUP to FTLDNS so it reloads the lists
    kill(Pid::from_raw(pid as libc::pid_t), Signal::SIGHUP).context(ErrorKind::ReloadDnsError)?;

    Ok(())
}
