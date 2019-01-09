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

extern crate base64;
extern crate regex;
extern crate rmp;
#[macro_use]
extern crate rocket;
extern crate rocket_cors;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate rust_embed;
extern crate failure;
extern crate failure_derive;
extern crate get_if_addrs;
extern crate hostname;
extern crate libc;
extern crate nix;
extern crate shmem;
extern crate tempfile;
extern crate toml;

pub use setup::start;

mod auth;
mod env;
mod ftl;
mod routes;
mod settings;
mod setup;
mod util;

#[cfg(test)]
mod testing;
