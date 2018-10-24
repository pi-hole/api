// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::{
    FtlClient, FtlCounters, FtlDomain, FtlOverTime, FtlQuery, FtlStrings, FtlUpstream, ShmLock,
    ShmLockGuard
};
use libc;
use shmem::{Array, Map, Object};
use std::{marker::PhantomData, ops::Deref};
use util::Error;

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
    Production {
        lock: ShmLock
    },
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
    /// Create a production instance of `FtlMemory`
    pub fn production() -> FtlMemory {
        FtlMemory::Production {
            lock: ShmLock::new()
        }
    }

    /// Get the FTL shared memory lock. The resulting [`ShmLockGuard`] is used
    /// to access the rest of shared memory.
    ///
    /// [`ShmLockGuard`]: ../shared_lock/enum.ShmLockGuard.html
    pub fn lock(&self) -> Result<ShmLockGuard, Error> {
        match self {
            FtlMemory::Production { lock } => lock.read(),
            #[cfg(test)]
            FtlMemory::Test { .. } => Ok(ShmLockGuard::Test)
        }
    }

    /// Get the FTL shared memory client data. The resulting trait object can
    /// dereference into `&[FtlClient]`.
    pub fn clients<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = [FtlClient]> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_CLIENTS)?)?
            ),
            #[cfg(test)]
            FtlMemory::Test { clients, .. } => Box::new(clients.as_slice())
        })
    }

    /// Get the FTL shared memory domain data. The resulting trait object can
    /// dereference into `&[FtlDomain]`.
    pub fn domains<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = [FtlDomain]> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_DOMAINS)?)?
            ),
            #[cfg(test)]
            FtlMemory::Test { domains, .. } => Box::new(domains.as_slice())
        })
    }

    /// Get the FTL shared memory overTime data. The resulting trait object can
    /// dereference into `&[FtlOverTime]`.
    pub fn over_time<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = [FtlOverTime]> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_OVERTIME)?)?
            ),
            #[cfg(test)]
            FtlMemory::Test { over_time, .. } => Box::new(over_time.as_slice())
        })
    }

    /// Get the FTL shared memory overTime client data. The resulting trait
    /// object can dereference into `&[libc::c_int]`.
    pub fn over_time_client<'lock>(
        &'lock self,
        client_id: usize,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = [libc::c_int]> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(
                // Load the shared memory
                Array::new(Object::open(format!(
                    "{}{}",
                    FTL_SHM_OVERTIME_CLIENT, client_id
                ))?)?
            ),
            #[cfg(test)]
            FtlMemory::Test {
                over_time_clients, ..
            } => Box::new(over_time_clients[client_id].as_slice())
        })
    }

    /// Get the FTL shared memory upstream data. The resulting trait object can
    /// dereference into `&[FtlUpstream]`.
    pub fn upstreams<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = [FtlUpstream]> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_FORWARDED)?)?
            ),
            #[cfg(test)]
            FtlMemory::Test { upstreams, .. } => Box::new(upstreams.as_slice())
        })
    }

    /// Get the FTL shared memory query data. The resulting trait object can
    /// dereference into `&[FtlQuery]`.
    pub fn queries<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = [FtlQuery]> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(
                // Load the shared memory
                Array::new(Object::open(FTL_SHM_QUERIES)?)?
            ),
            #[cfg(test)]
            FtlMemory::Test { queries, .. } => Box::new(queries.as_slice())
        })
    }

    /// Get the FTL shared memory string data
    pub fn strings<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<FtlStrings<'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => {
                FtlStrings::Production(Array::new(Object::open(FTL_SHM_STRINGS)?)?, PhantomData)
            }
            #[cfg(test)]
            FtlMemory::Test { strings, .. } => FtlStrings::Test(&strings)
        })
    }

    /// Get the FTL shared memory counters data. The resulting trait object can
    /// dereference into `&FtlCounters`.
    pub fn counters<'lock>(
        &'lock self,
        _lock_guard: &ShmLockGuard<'lock>
    ) -> Result<Box<dyn Deref<Target = FtlCounters> + 'lock>, Error> {
        Ok(match self {
            FtlMemory::Production { .. } => Box::new(Map::new(Object::open(FTL_SHM_COUNTERS)?)?),
            #[cfg(test)]
            FtlMemory::Test { counters, .. } => Box::new(counters)
        })
    }
}
