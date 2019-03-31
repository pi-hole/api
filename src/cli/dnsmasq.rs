// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Generate the Dnsmasq Config From CLI
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Config, Env},
    routes::settings::restart_dns,
    settings::generate_dnsmasq_config,
    util::Error
};

/// Generate the dnsmasq config using [`generate_dnsmasq_config`]. Dnsmasq (FTL)
/// will be restarted in the process of applying the changes. This should be
/// called when handling the [`GenerateDnsmasq`] command on the CLI.
///
/// [`generate_dnsmasq_config`]:
/// ../../settings/dnsmasq/fn.generate_dnsmasq_config.html
/// [`GenerateDnsmasq`]: ../args/enum.CliCommand.html#variant.GenerateDnsmasq
pub fn generate_dnsmasq_cli() -> Result<(), Error> {
    let config = Config::load()?;
    let env = Env::Production(config);

    println!("Generating dnsmasq config...");

    generate_dnsmasq_config(&env)?;
    restart_dns(&env)?;

    println!("Done");

    Ok(())
}
