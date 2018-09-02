// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::{Env, PiholeFile};
use shmem::{self, Array, Object};
use std::collections::HashMap;
use std::ops::Deref;
use util::{Error, ErrorKind};
use ftl::{FtlClient, FtlStrings};

/// A wrapper for accessing FTL's shared memory.
///
/// - Production mode connects to the real FTL shared memory.
/// - Test mode uses the associated test data to mock FTL's shared memory.
pub enum FtlMemory {
    Production,
    Test {
        clients: Vec<FtlClient>,
        strings: HashMap<usize, String>
    }
}

impl FtlMemory {
    /// Get the FTL shared memory client data. The resulting trait object owns
    /// the client data and can dereference into `&[FtlClient]`.
    pub fn clients<'test>(
        &'test self,
        env: &Env
    ) -> Result<Box<dyn Deref<Target = [FtlClient]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(
                    Object::open(env.file_location(PiholeFile::FtlShmClients))
                        .map_err(from_shmem_error)?
                ).map_err(from_shmem_error)?
            ),
            FtlMemory::Test { clients, .. } => Box::new(clients.as_slice())
        })
    }

    /// Get the FTL shared memory string data
    pub fn strings(&self, env: &Env) -> Result<FtlStrings, Error> {
        Ok(match self {
            FtlMemory::Production => FtlStrings::Production(
                Array::new(
                    Object::open(env.file_location(PiholeFile::FtlShmStrings))
                        .map_err(from_shmem_error)?
                ).map_err(from_shmem_error)?
            ),
            FtlMemory::Test { strings, .. } => FtlStrings::Test(&strings)
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
