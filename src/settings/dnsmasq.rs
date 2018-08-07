// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Dnsmasq Configuration Generator
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::{Env, PiholeFile};
use failure::{Fail, ResultExt};
use settings::{ConfigEntry, SetupVarsEntry};
use std::io::{self, BufWriter, Write};
use util::{Error, ErrorKind};

const DNSMASQ_HEADER: &str = "\
################################################################
#        THIS FILE IS AUTOMATICALLY WRITTEN BY PI-HOLE.        #
#          ANY CHANGES MADE TO THIS FILE WILL BE LOST          #
#                                                              #
#   ANY OTHER CHANGES MUST BE MADE IN A SEPARATE CONFIG FILE   #
#                OR IN /etc/dnsmasq.conf                       #
################################################################

localise-queries
";

/// Generate a dnsmasq config based off of SetupVars.
pub fn generate_dnsmasq_config(env: &Env) -> Result<(), Error> {
    let mut config_file = BufWriter::new(env.write_file(PiholeFile::DnsmasqConfig, false)?);

    let apply_context = |error: io::Error| {
        let context =
            ErrorKind::FileWrite(env.file_location(PiholeFile::DnsmasqConfig).to_owned());
        error.context(context.into())
    };

    // Write header
    config_file
        .write_all(DNSMASQ_HEADER.as_bytes())
        .map_err(apply_context)?;

    // Add upstream DNS servers
    for i in 1.. {
        let dns = SetupVarsEntry::PiholeDns(i).read(env)?;

        // When the setting is empty, we are finished adding servers
        if dns.is_empty() {
            break;
        }

        writeln!(config_file, "server={}", dns).map_err(apply_context)?;
    }

    // Add blocklist and blacklist if blocking is enabled
    if SetupVarsEntry::Enabled
        .read(env)?
        .parse::<bool>()
        .context(ErrorKind::InvalidSettingValue)?
    {
        config_file
            .write_all(b"addn-hosts=/etc/pihole/gravity.list\n")
            .map_err(apply_context)?;
        config_file
            .write_all(b"addn-hosts=/etc/pihole/black.list\n")
            .map_err(apply_context)?;
    }

    // Always add local.list after the blocklists
    config_file
        .write_all(b"addn-hosts=/etc/pihole/local.list\n")
        .map_err(apply_context)?;

    // Add various DNS settings if enabled
    if SetupVarsEntry::DnsFqdnRequired
        .read(env)?
        .parse::<bool>()
        .context(ErrorKind::InvalidSettingValue)?
    {
        config_file
            .write_all(b"domain-needed\n")
            .map_err(apply_context)?;
    }

    if SetupVarsEntry::DnsBogusPriv
        .read(env)?
        .parse::<bool>()
        .context(ErrorKind::InvalidSettingValue)?
    {
        config_file
            .write_all(b"bogus-priv\n")
            .map_err(apply_context)?;
    }

    if SetupVarsEntry::Dnssec
        .read(env)?
        .parse::<bool>()
        .context(ErrorKind::InvalidSettingValue)?
    {
        config_file.write_all(
            b"dnssec\n\
            trust-anchor=.,19036,8,2,49AAC11D7B6F6446702E54A1607371607A1A41855200FD2CE1CDDE32F24E8FB5\n\
            trust-anchor=.,20326,8,2,E06D44B80B8F1D39A95C0B0D7C65D08458E880409BBC683457104237C7F8EC8D\n"
        ).map_err(apply_context)?;
    }

    let host_record = SetupVarsEntry::HostRecord.read(env)?;
    if !host_record.is_empty() {
        writeln!(config_file, "host-record={}", host_record).map_err(apply_context)?;
    }

    match SetupVarsEntry::DnsmasqListening.read(env)?.as_str() {
        "all" => config_file
            .write_all(b"except-interface=nonexisting\n")
            .map_err(apply_context)?,
        "local" => config_file
            .write_all(b"local-service")
            .map_err(apply_context)?,
        "single" | _ => {
            writeln!(
                config_file,
                "interface={}",
                SetupVarsEntry::PiholeInterface.read(env)?
            ).map_err(apply_context)?;
        }
    }

    if SetupVarsEntry::ConditionalForwarding
        .read(env)?
        .parse::<bool>()
        .context(ErrorKind::InvalidSettingValue)?
    {
        let ip = SetupVarsEntry::ConditionalForwardingIp.read(env)?;

        writeln!(
            config_file,
            "server=/{}/{}\nserver=/{}/{}",
            SetupVarsEntry::ConditionalForwardingDomain.read(env)?,
            ip,
            SetupVarsEntry::ConditionalForwardingReverse.read(env)?,
            ip
        ).map_err(apply_context)?;
    }

    Ok(())
}
