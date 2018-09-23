// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Setting Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod common;
mod dns;
mod get_dhcp;
mod get_ftl;
mod get_ftldb;
mod get_network;

pub use self::{common::*, dns::*, get_dhcp::*, get_ftl::*, get_ftldb::*, get_network::*};
