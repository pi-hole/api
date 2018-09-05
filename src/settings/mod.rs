// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Setting Specifications For SetupVars & FTL Configuration Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod dnsmasq;
mod entries;
mod privacy_level;
mod value_type;

pub use self::dnsmasq::generate_dnsmasq_config;
pub use self::entries::{ConfigEntry, FtlConfEntry, SetupVarsEntry};
pub use self::privacy_level::FtlPrivacyLevel;
pub use self::value_type::ValueType;
