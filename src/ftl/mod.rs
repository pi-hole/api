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
mod shared_memory;
mod memory_model;

pub use self::connection::{FtlConnection, FtlConnectionType};
pub use self::shared_memory::FtlMemory;
pub use self::memory_model::{FtlClient, FtlStrings};
