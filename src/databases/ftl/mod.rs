// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Support
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod model;
mod schema;

pub use self::{model::*, schema::*};
