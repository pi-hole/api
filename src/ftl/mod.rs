// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Utilities
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod lock_thread;
mod memory_model;
mod shared_lock;
mod shared_memory;
mod socket;

pub use self::{
    memory_model::*,
    shared_lock::{ShmLock, ShmLockGuard},
    shared_memory::FtlMemory,
    socket::{FtlConnection, FtlConnectionType}
};
