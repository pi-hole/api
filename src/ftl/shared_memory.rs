// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::{FtlClient, FtlCounters, FtlDomain, FtlOverTime, FtlQuery, FtlStrings, FtlUpstream};
use libc;
use shmem::{self, Array, Map, Object};
use std::marker::PhantomData;
use std::ops::Deref;
use util::{Error, ErrorKind};

#[cfg(test)]
use std::collections::HashMap;

const FTL_SHM_CLIENTS: &str = "/FTL-clients";
const FTL_SHM_DOMAINS: &str = "/FTL-domains";
const FTL_SHM_FORWARDED: &str = "/FTL-forwarded";
const FTL_SHM_OVERTIME: &str = "/FTL-overTime";
const FTL_SHM_OVERTIME_CLIENT: &str = "/FTL-client-{}";
const FTL_SHM_QUERIES: &str = "/FTL-queries";
const FTL_SHM_STRINGS: &str = "/FTL-strings";
const FTL_SHM_COUNTERS: &str = "/FTL-counters";

/// A wrapper for accessing FTL's shared memory.
///
/// - Production mode connects to the real FTL shared memory.
/// - Test mode uses the associated test data to mock FTL's shared memory.
pub enum FtlMemory {
    Production,
    #[cfg(test)]
    Test {
        clients: Vec<FtlClient>,
        domains: Vec<FtlDomain>,
        over_time: Vec<FtlOverTime>,
        over_time_clients: Vec<Vec<libc::c_int>>,
        upstreams: Vec<FtlUpstream>,
        queries: Vec<FtlQuery>,
        strings: HashMap<usize, String>,
        counters: FtlCounters
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
            #[cfg(test)]
            FtlMemory::Test { clients, .. } => Box::new(clients.as_slice())
        })
    }

    /// Get the FTL shared memory domain data. The resulting trait object can
    /// dereference into `&[FtlDomain]`.
    pub fn domains<'test>(
        &'test self
    ) -> Result<Box<dyn Deref<Target = [FtlDomain]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_DOMAINS).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            #[cfg(test)]
            FtlMemory::Test { domains, .. } => Box::new(domains.as_slice())
        })
    }

    /// Get the FTL shared memory overTime data. The resulting trait object can
    /// dereference into `&[FtlOverTime]`.
    pub fn over_time<'test>(
        &'test self
    ) -> Result<Box<dyn Deref<Target = [FtlOverTime]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_OVERTIME).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            #[cfg(test)]
            FtlMemory::Test { over_time, .. } => Box::new(over_time.as_slice())
        })
    }

    /// Get the FTL shared memory overTime client data. The resulting trait
    /// object can dereference into `&[libc::c_int]`.
    pub fn over_time_client<'test>(
        &'test self,
        client_id: usize
    ) -> Result<Box<dyn Deref<Target = [libc::c_int]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(
                    Object::open(format!("{}{}", FTL_SHM_OVERTIME_CLIENT, client_id))
                        .map_err(from_shmem_error)?
                ).map_err(from_shmem_error)?
            ),
            #[cfg(test)]
            FtlMemory::Test {
                over_time_clients, ..
            } => Box::new(over_time_clients[client_id].as_slice())
        })
    }

    /// Get the FTL shared memory upstream data. The resulting trait object can
    /// dereference into `&[FtlUpstream]`.
    pub fn upstreams<'test>(
        &'test self
    ) -> Result<Box<dyn Deref<Target = [FtlUpstream]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_FORWARDED).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            #[cfg(test)]
            FtlMemory::Test { upstreams, .. } => Box::new(upstreams.as_slice())
        })
    }

    /// Get the FTL shared memory query data. The resulting trait object can
    /// dereference into `&[FtlQuery]`.
    pub fn queries<'test>(
        &'test self
    ) -> Result<Box<dyn Deref<Target = [FtlQuery]> + 'test>, Error> {
        Ok(match self {
            FtlMemory::Production => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_QUERIES).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?
            ),
            #[cfg(test)]
            FtlMemory::Test { queries, .. } => Box::new(queries.as_slice())
        })
    }

    /// Get the FTL shared memory string data
    pub fn strings(&self) -> Result<FtlStrings, Error> {
        Ok(match self {
            FtlMemory::Production => FtlStrings::Production(
                Array::new(Object::open(FTL_SHM_STRINGS).map_err(from_shmem_error)?)
                    .map_err(from_shmem_error)?,
                PhantomData
            ),
            #[cfg(test)]
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
            #[cfg(test)]
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
