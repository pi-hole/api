// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Root Library File
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate regex;
extern crate rmp;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rust_embed;
extern crate failure;
extern crate toml;
#[macro_use]
extern crate failure_derive;
extern crate hostname;
extern crate tempfile;

pub use setup::*;

mod auth;
mod config;
mod config_files;
mod ftl;
mod routes;
mod setup;
mod setup_vars;
mod util;

#[cfg(test)]
mod testing;
