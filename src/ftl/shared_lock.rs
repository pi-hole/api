// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Lock
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use failure::ResultExt;
use libc::{pthread_rwlock_rdlock, pthread_rwlock_t, pthread_rwlock_unlock, pthread_rwlock_wrlock};
use nix::errno::Errno;
use shmem::Map;
use util::{Error, ErrorKind};

/// A lock for shared memory for coordinating shared memory access with FTL.
pub enum ShmLock {
    Production {
        lock: Map<pthread_rwlock_t>
    },
    #[cfg(test)]
    Test
}

impl ShmLock {
    /// Acquire a read lock on the shared memory. It will last as long as the
    /// guard (return value) lives.
    pub fn read(&mut self) -> Result<ShmLockGuard, Error> {
        match self {
            ShmLock::Production { lock } => {
                let lock: &mut pthread_rwlock_t = lock;

                Errno::result(unsafe { pthread_rwlock_rdlock(lock) })
                    .context(ErrorKind::SharedMemoryLock)?;

                Ok(ShmLockGuard::Production { lock })
            }
            #[cfg(test)]
            ShmLock::Test => Ok(ShmLockGuard::Test)
        }
    }

    /// Acquire a write lock on the shared memory. It will last as long as the
    /// guard (return value) lives.
    #[allow(unused)]
    pub fn write(&mut self) -> Result<ShmLockGuard, Error> {
        match self {
            ShmLock::Production { lock } => {
                let lock: &mut pthread_rwlock_t = lock;

                Errno::result(unsafe { pthread_rwlock_wrlock(lock) })
                    .context(ErrorKind::SharedMemoryLock)?;

                Ok(ShmLockGuard::Production { lock })
            }
            #[cfg(test)]
            ShmLock::Test => Ok(ShmLockGuard::Test)
        }
    }
}

/// A RAII type lock guard which keeps the lock active until it is dropped.
pub enum ShmLockGuard<'lock> {
    Production {
        lock: &'lock mut pthread_rwlock_t
    },
    #[cfg(test)]
    Test
}

impl<'lock> Drop for ShmLockGuard<'lock> {
    fn drop(&mut self) {
        match self {
            ShmLockGuard::Production { lock } => unsafe {
                pthread_rwlock_unlock(*lock);
            },
            #[cfg(test)]
            ShmLockGuard::Test => ()
        }
    }
}
