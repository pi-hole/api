// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Handle the CLI Arguments
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    cli::args::{CliArgs, CliCommand},
    setup::start,
    util::Error
};
use structopt::StructOpt;

/// Parse the CLI arguments and execute the command. If there are no commands,
/// start the API.
pub fn handle_cli() -> Result<(), Error> {
    // Parse the command line arguments
    let args: CliArgs = CliArgs::from_args();

    // Check for commands
    match args.command {
        // Execute the command
        Some(command) => match command {
            CliCommand::Version => println!("{}", get_version()),
            CliCommand::Branch => println!("{}", get_branch()),
            CliCommand::Hash => println!("{}", get_hash())
        },
        // No command given, start the API
        None => start()?
    }

    Ok(())
}

pub fn get_version() -> &'static str {
    env!("GIT_VERSION")
}

fn get_branch() -> &'static str {
    env!("GIT_BRANCH")
}

fn get_hash() -> &'static str {
    env!("GIT_HASH").get(0..7).unwrap_or_default()
}
