// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Root Library File
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate rust_embed;

pub use crate::setup::start;

mod auth;
mod env;
mod ftl;
mod routes;
mod settings;
mod setup;
mod util;

#[cfg(test)]
mod testing;
