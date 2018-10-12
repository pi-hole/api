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
use libc::{pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock};
use nix::errno::Errno;
use shmem::Map;
use std::{
    ops::DerefMut,
    sync::{Condvar, Mutex}
};
use util::{Error, ErrorKind};

/// A lock for coordinating shared memory access with FTL. It locks a mutex in
/// shared memory, and while holding the lock it distributes read locks. If it
/// detects that FTL is waiting for a lock on the shared mutex, it will stop
/// distributing read locks until FTL gets the lock back.
pub struct ShmLock {
    lock_count_lock: Mutex<usize>
}

impl ShmLock {
    /// Create a new `ShmLock` with a lock count of zero.
    pub fn new() -> ShmLock {
        ShmLock {
            lock_count_lock: Mutex::new(0)
        }
    }

    /// Acquire a read lock on the shared memory. It will last as long as the
    /// guard (return value) lives. The shm_lock parameter is taken in case
    /// shared memory needs to be locked or unlocked (for the first lock to be
    /// taken, or the last lock to be released).
    pub fn read(&self, mut shm_lock: Map<pthread_mutex_t>) -> Result<ShmLockGuard, Error> {
        let mut lock_count = self.lock_count_lock.lock().unwrap_or_else(|e| {
            // The lock was poisoned, which means a thread panicked while
            // holding the lock. This most likely could have happened when
            // trying to acquire the shared memory lock, or when unlocking the
            // shared memory lock.

            // Get the MutexGuard from the poisoned lock
            let mut lock_count = e.into_inner();

            // If the lock_count is zero, it failed while acquiring the lock and
            // no action needs to be taken.
            // If the lock_count is non-zero, it failed while holding the lock,
            // and so the lock_count should be decremented because the thread no
            // longer holds the lock.
            if *lock_count != 0 {
                *lock_count -= 1;
            }

            lock_count
        });

        // Only acquire a lock if there are no active read locks
        if *lock_count == 0 {
            // Try to acquire a lock
            Errno::result(unsafe { pthread_mutex_lock(shm_lock.deref_mut()) })
                .context(ErrorKind::SharedMemoryLock)?;
        }

        // Increment the lock count
        *lock_count += 1;

        Ok(ShmLockGuard::Production {
            lock: self,
            shm_lock
        })
    }
}

/// A RAII type lock guard which keeps the lock active until it is dropped.
pub enum ShmLockGuard<'lock> {
    Production {
        lock: &'lock ShmLock,
        shm_lock: Map<pthread_mutex_t>
    },
    #[cfg(test)]
    Test
}

impl<'lock> Drop for ShmLockGuard<'lock> {
    fn drop(&mut self) {
        match self {
            ShmLockGuard::Production {
                lock,
                ref mut shm_lock
            } => {
                let mut lock_count = lock.lock_count_lock.lock().unwrap();

                // Check if we should release the mutex
                if *lock_count == 1 {
                    unsafe {
                        pthread_mutex_unlock(shm_lock.deref_mut());
                    }
                }

                // Decrement the lock count
                *lock_count -= 1;
            }
            #[cfg(test)]
            ShmLockGuard::Test => ()
        }
    }
}
