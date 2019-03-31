// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// CLI Arguments and Options
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::cli::handler::get_version;
use structopt::{clap::AppSettings, StructOpt};

/// This defines the arguments that the CLI can be given
///
/// `AppSettings::VersionlessSubcommands` will remove the `-V` version flag from
/// sub-commands. All sub-commands in this project have the same version.
#[derive(StructOpt)]
#[structopt(
    name = "pihole-API",
    about = "Work in progress HTTP API for Pi-hole.",
    author = "",
    raw(version = "get_version()"),
    raw(global_setting = "AppSettings::VersionlessSubcommands")
)]
pub struct CliArgs {
    #[structopt(subcommand)]
    pub command: Option<CliCommand>
}

/// The commands that the CLI handles
#[derive(StructOpt)]
pub enum CliCommand {
    /// Prints version information
    #[structopt(name = "version", author = "", raw(version = "get_version()"))]
    Version,
    /// Prints branch
    #[structopt(name = "branch", author = "", raw(version = "get_version()"))]
    Branch,
    /// Prints git hash
    #[structopt(name = "hash", author = "", raw(version = "get_version()"))]
    Hash
}
