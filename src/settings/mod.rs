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

pub use self::{
    dnsmasq::generate_dnsmasq_config,
    entries::{ConfigEntry, FtlConfEntry, SetupVarsEntry},
    privacy_level::FtlPrivacyLevel,
    value_type::ValueType
};
