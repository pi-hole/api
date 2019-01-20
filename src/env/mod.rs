// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Configuration
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod config;
mod env_impl;
mod file;

pub use self::{config::Config, env_impl::Env, file::PiholeFile};
