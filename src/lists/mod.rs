// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// List Service and Repository (Whitelist, Blacklist, Regexlist)
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod repository;
mod service;

pub use self::{repository::*, service::*};
