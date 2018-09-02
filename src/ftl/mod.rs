// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Utilities
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod connection;
mod memory_model;
mod shared_memory;

pub use self::connection::{FtlConnection, FtlConnectionType};
pub use self::memory_model::{FtlClient, FtlStrings};
pub use self::shared_memory::FtlMemory;
