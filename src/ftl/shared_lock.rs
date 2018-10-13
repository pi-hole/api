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
use ftl::memory_model::FtlLock;
use libc::{pthread_cond_wait, pthread_mutex_lock, pthread_mutex_unlock};
use nix::errno::Errno;
use shmem::Map;
use std::sync::Mutex;
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
    pub fn read(&self, mut shm_lock: Map<FtlLock>) -> Result<ShmLockGuard, Error> {
        // Flag which marks if the shm lock was acquired via condition variable
        let mut shm_lock_acquired = false;

        // Check if FTL is waiting for the lock
        if shm_lock.ftl_waiting_for_lock {
            // Wait on the condition variable
            while shm_lock.ftl_waiting_for_lock {
                Errno::result(unsafe {
                    pthread_cond_wait(&mut shm_lock.cond_var, &mut shm_lock.lock)
                }).context(ErrorKind::SharedMemoryLock)?;
                shm_lock_acquired = true;
            }
        }

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

        // Only acquire a lock if we didn't get it via the condition variable
        // and there are no active read locks.
        if !shm_lock_acquired && *lock_count == 0 {
            // Try to acquire a lock
            Errno::result(unsafe { pthread_mutex_lock(&mut shm_lock.lock) })
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
        shm_lock: Map<FtlLock>
    },
    #[cfg(test)]
    Test
}

impl<'lock> Drop for ShmLockGuard<'lock> {
    fn drop(&mut self) {
        match self {
            ShmLockGuard::Production { lock, shm_lock } => {
                let mut lock_count = lock.lock_count_lock.lock().unwrap();

                // Check if we should release the mutex
                if *lock_count == 1 {
                    unsafe {
                        pthread_mutex_unlock(&mut shm_lock.lock);
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
