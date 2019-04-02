// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// CLI Handling
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod args;
mod dnsmasq;
mod handler;

pub use self::handler::handle_cli;
