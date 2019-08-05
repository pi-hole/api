// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Services (and supporting code) of the API
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

#[macro_use]
mod service;

pub mod lists;

pub use self::service::*;
