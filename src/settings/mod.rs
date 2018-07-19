// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Setting Specifications For SetupVars & FTL Configuration Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod entries;
mod value_type;

pub use self::entries::*;
pub use self::value_type::*;
