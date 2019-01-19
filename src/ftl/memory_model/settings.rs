// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Settings Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use libc;

/// The settings structure used to share version information and other settings
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FtlSettings {
    pub version: libc::c_int
}

impl Default for FtlSettings {
    fn default() -> Self {
        FtlSettings { version: 0 }
    }
}
