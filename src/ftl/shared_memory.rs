// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::{FtlClient, FtlCounters, FtlStrings};
use shmem::{self, Array, Map, Object};
use std::collections::HashMap;
use std::ops::Deref;
use util::{Error, ErrorKind};

const FTL_SHM_CLIENTS: &str = "/FTL-clients";
const FTL_SHM_DOMAINS: &str = "/FTL-domains";
const FTL_SHM_FORWARDED: &str = "/FTL-forwarded";
const FTL_SHM_QUERIES: &str = "/FTL-queries";
const FTL_SHM_STRINGS: &str = "/FTL-strings";
const FTL_SHM_COUNTERS: &str = "/FTL-counters";

/// A wrapper for accessing FTL's shared memory.
///
/// - Production mode connects to the real FTL shared memory.
/// - Test mode uses the associated test data to mock FTL's shared memory.
pub enum FtlMemory {
    Production,
    Test {
        clients: Vec<FtlClient>,
        strings: HashMap<usize, String>,
        counters: FtlCounters
    }
}

impl Default for FtlMemory {
    fn default() -> Self {
        FtlMemory::Test {
            clients: Vec::new(),
            strings: HashMap::new(),
            counters: FtlCounters::default()
        }
    }
}

impl FtlMemory {
    /// Get the FTL shared memory client data. The resulting trait object can
    /// dereference into `&[FtlClient]`.
    pub fn clients<'test>(
        &'test self
    ) -> Result<Box<dyn Deref<Target = [FtlClient]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_CLIENTS).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            FtlMemory::Test { clients, .. } => Box::new(clients.as_slice())
        })
    }

    /// Get the FTL shared memory string data
    pub fn strings(&self) -> Result<FtlStrings, Error> {
        Ok(match self {
            FtlMemory::Production => FtlStrings::Production(
                Array::new(Object::open(FTL_SHM_STRINGS).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            FtlMemory::Test { strings, .. } => FtlStrings::Test(&strings)
        })
    }

    /// Get the FTL shared memory counters data. The resulting trait object can
    /// dereference into `&FtlCounters`.
    pub fn counters<'test>(
        &'test self
    ) -> Result<Box<dyn Deref<Target = FtlCounters> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                Map::new(Object::open(FTL_SHM_COUNTERS).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            FtlMemory::Test { counters, .. } => Box::new(counters)
        })
    }
}

/// Converts `shmem::Error` into [`ErrorKind::SharedMemoryOpen`]. See the
/// comment on [`ErrorKind::SharedMemoryOpen`] for more information.
///
/// [`ErrorKind::SharedMemoryOpen`]:
/// ../../util/enum.ErrorKind.html#variant.SharedMemoryOpen
fn from_shmem_error(e: shmem::Error) -> ErrorKind {
    ErrorKind::SharedMemoryOpen(format!("{:?}", e))
}
