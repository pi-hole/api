/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Root Library File
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate regex;
extern crate rmp;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub use setup::*;

mod util;
mod ftl;
mod stats;
mod dns;
mod web;
mod setup;
mod config;

#[cfg(test)]
mod testing;
