// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Lock Handler Thread
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::memory_model::FtlLock, util::Error};
use libc::{self, pthread_mutex_lock, pthread_mutex_unlock};
use shmem::{Map, Object};
use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration
};

/// The filename of the shared memory, used to connect to the shared memory
/// lock.
const FTL_SHM_LOCK: &str = "/FTL-lock";

/// A type alias for the type of requests sent to the lock thread.
pub type LockRequest = (RequestType, Sender<LockResponse>);

/// A type alias for the type of responses sent from the lock thread.
pub type LockResponse = Result<libc::c_int, Error>;

/// Try to open the shared memory lock
fn open_shm_lock() -> Result<Map<FtlLock>, Error> {
    Ok(Map::new(Object::open(FTL_SHM_LOCK)?)?)
}

/// The type of action that the lock thread is requested to perform.
#[derive(Debug, PartialEq)]
pub enum RequestType {
    Lock,
    Unlock
}

/// The lock thread handler. This thread takes in lock requests and keeps track
/// of open read locks.
pub struct LockThread {
    pub(self) lock_count: usize,
    pub(self) wait_queue: VecDeque<Sender<LockResponse>>
}

impl LockThread {
    /// Create a LockThread. Note that this does not spawn a thread. To finish
    /// setup, run [`handle_requests`] in a new thread.
    ///
    /// [`handle_requests`]: #method.handle_requests
    pub fn new() -> LockThread {
        LockThread {
            lock_count: 0,
            wait_queue: VecDeque::new()
        }
    }

    /// Handle incoming lock requests, retrieved from `request_receiver`. The
    /// shared memory lock is only opened when a request is received.
    pub fn handle_requests(&mut self, request_receiver: Receiver<LockRequest>) {
        for (request_type, response_sender) in request_receiver.iter() {
            let mut shm_lock = match open_shm_lock() {
                Ok(lock) => lock,
                Err(e) => {
                    response_sender.send(Err(e)).unwrap();
                    continue;
                }
            };

            match request_type {
                RequestType::Lock => self.lock(&mut shm_lock, response_sender),
                RequestType::Unlock => self.unlock(&mut shm_lock, response_sender)
            }
        }
    }

    /// Try to acquire the shared memory lock. If FTL is waiting, the lock
    /// request will be put onto the wait queue for later. If we already have
    /// the shared memory lock, the lock count will simply be incremented.
    pub(self) fn lock(&mut self, shm_lock: &mut FtlLock, sender: Sender<LockResponse>) {
        if shm_lock.ftl_waiting_for_lock {
            if self.lock_count > 0 {
                // If we own the lock, defer to FTL
                self.wait_queue.push_back(sender);
                return;
            } else {
                // If we don't own the lock, FTL should get it very soon (or it
                // died and we can take the lock)
                LockThread::wait_for_ftl(shm_lock);
            }
        }

        // Check if we need to lock the shared memory lock
        if self.lock_count == 0 {
            let ret = unsafe { pthread_mutex_lock(&mut shm_lock.lock) };

            // Check for a lock error
            if ret != 0 {
                sender.send(Ok(ret)).unwrap();
                return;
            }
        }

        self.lock_count += 1;
        sender.send(Ok(0)).unwrap();
    }

    /// Remove a lock from the lock count and if necessary unlock the shared
    /// memory lock. If unlocking the shared locks succeeds, check if FTL needs
    /// the lock and wait until it acquires it, then handle the lock requests
    /// which were put on hold.
    pub(self) fn unlock(&mut self, shm_lock: &mut FtlLock, sender: Sender<LockResponse>) {
        self.lock_count -= 1;

        // If there is at least one lock still, then we don't need to do
        // anything with the shared memory lock or queued lock requests.
        if self.lock_count != 0 {
            sender.send(Ok(0)).unwrap();
            return;
        }

        // There are no more read locks, so unlock the shared memory lock.
        let ret = unsafe { pthread_mutex_unlock(&mut shm_lock.lock) };

        // Check for an unlock error
        if ret != 0 {
            sender.send(Ok(ret)).unwrap();
            return;
        } else {
            sender.send(Ok(0)).unwrap();
        }

        // If FTL is waiting for the lock, let it get the lock before going
        // through the queued lock requests.
        LockThread::wait_for_ftl(shm_lock);

        // Only go through the lock requests that were in the queue initially
        // (not those which might be added in the process)
        let queued_senders: Vec<_> = self.wait_queue.drain(..).collect();
        for queued_sender in queued_senders {
            self.lock(shm_lock, queued_sender);
        }
    }

    /// Wait for FTL to take the lock if it signaled it needs it. If it doesn't
    /// take the lock within a timeout (10 seconds), the signal will be turned
    /// off. Either way, when the function returns it is safe to take the lock.
    fn wait_for_ftl(shm_lock: &mut FtlLock) {
        if cfg!(test) {
            // When testing, do not wait for a timeout
            shm_lock.ftl_waiting_for_lock = false;
            return;
        }

        let mut ftl_wait_count = 0;
        while shm_lock.ftl_waiting_for_lock {
            // Sleep for 1 millisecond
            thread::sleep(Duration::new(0, 1000000));
            ftl_wait_count += 1;

            // If FTL is taking longer than ten seconds to take the lock, assume
            // something is wrong with it (perhaps it crashed) and stop waiting
            // for it to take the lock.
            if ftl_wait_count == 10000 {
                shm_lock.ftl_waiting_for_lock = false;
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ftl::{lock_thread::LockThread, memory_model::FtlLock};
    use libc::{
        pthread_mutex_destroy, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_trylock,
        pthread_mutex_unlock, EBUSY, PTHREAD_MUTEX_INITIALIZER
    };
    use std::sync::mpsc::channel;

    /// Lock a mutex
    fn lock_mutex(mutex: &mut pthread_mutex_t) {
        assert_eq!(unsafe { pthread_mutex_lock(mutex) }, 0);
    }

    /// Unlock a mutex
    fn unlock_mutex(mutex: &mut pthread_mutex_t) {
        assert_eq!(unsafe { pthread_mutex_unlock(mutex) }, 0);
    }

    /// Destroy a pthread mutex
    fn destroy_lock(mut lock: pthread_mutex_t) {
        assert_eq!(unsafe { pthread_mutex_destroy(&mut lock) }, 0);
    }

    /// Check that the mutex is locked and lock count incremented when getting
    /// the first lock
    #[test]
    fn first_lock() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: false
        };
        let (sender, receiver) = channel();

        assert_eq!(lock_thread.lock_count, 0);

        lock_thread.lock(&mut ftl_lock, sender);

        assert_eq!(lock_thread.lock_count, 1);

        // Check if the mutex was locked by attempting to lock it (returns EBUSY
        // if it was locked, or 0 if it was unlocked)
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, EBUSY);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }

    /// Check that the lock count is simply incremented if there already exist
    /// other read locks. The mutex should not be locked.
    #[test]
    fn second_lock() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: false
        };
        let (sender, receiver) = channel();

        lock_thread.lock_count = 1;

        lock_thread.lock(&mut ftl_lock, sender);

        assert_eq!(lock_thread.lock_count, 2);

        // The mutex was not locked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, 0);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }

    /// If the lock is not held by API and FTL signals that it needs the lock,
    /// API will let FTL take the lock. If FTL does not take the lock within 10
    /// seconds, API will turn off the signal and take the lock (FTL probably
    /// crashed while waiting for the lock). Note that when testing, the timeout
    /// is not implemented and API will simply seize the lock.
    #[test]
    fn ftl_waiting_lock_not_held() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: true
        };
        let (sender, receiver) = channel();

        assert_eq!(lock_thread.lock_count, 0);

        lock_thread.lock(&mut ftl_lock, sender);

        assert_eq!(lock_thread.lock_count, 1);

        // The FTL wait signal was turned off
        assert_eq!(ftl_lock.ftl_waiting_for_lock, false);

        // The mutex was locked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, EBUSY);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }

    /// Check that the lock request is put in the wait queue when FTL is
    /// waiting for the lock and the SHM lock is held by API. The mutex should
    /// not be locked.
    #[test]
    fn ftl_waiting_while_lock_held() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: true
        };
        let (sender, receiver) = channel();

        lock_thread.lock_count = 1;

        assert_eq!(lock_thread.wait_queue.len(), 0);

        lock_thread.lock(&mut ftl_lock, sender);

        // The lock count should not change, because the lock request has been
        // deferred
        assert_eq!(lock_thread.lock_count, 1);

        // The lock request has been put on the wait queue
        assert_eq!(lock_thread.wait_queue.len(), 1);

        // The mutex was not locked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, 0);
        unlock_mutex(&mut ftl_lock.lock);

        // No response has been sent
        assert!(receiver.try_recv().is_err());

        destroy_lock(ftl_lock.lock);
    }

    /// Check that the shared memory lock is unlocked after the last read lock
    /// is dead.
    #[test]
    fn last_unlock() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: false
        };
        let (sender, receiver) = channel();

        lock_thread.lock_count = 1;

        // Lock the mutex
        lock_mutex(&mut ftl_lock.lock);

        lock_thread.unlock(&mut ftl_lock, sender);

        assert_eq!(lock_thread.lock_count, 0);

        // The mutex was unlocked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, 0);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }

    /// Only decrement the lock count when unlocking if there are still more
    /// open read locks. The mutex should not be unlocked.
    #[test]
    fn unlock_with_multiple_locks() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: false
        };
        let (sender, receiver) = channel();

        lock_thread.lock_count = 2;

        // Lock the mutex
        lock_mutex(&mut ftl_lock.lock);

        lock_thread.unlock(&mut ftl_lock, sender);

        assert_eq!(lock_thread.lock_count, 1);

        // The mutex was not unlocked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, EBUSY);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }

    /// Make sure FTL gets the lock after unlocking, or if it has died, reset
    /// the wait signal.
    #[test]
    fn unlock_with_ftl_waiting() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: true
        };
        let (sender, receiver) = channel();

        lock_thread.lock_count = 1;

        // Lock the mutex
        lock_mutex(&mut ftl_lock.lock);

        lock_thread.unlock(&mut ftl_lock, sender);

        assert_eq!(lock_thread.lock_count, 0);

        // The FTL wait signal was removed (either by FTL or API)
        assert_eq!(ftl_lock.ftl_waiting_for_lock, false);

        // The mutex was unlocked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, 0);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }

    /// Handle queued lock requests after unlocking the shared memory lock.
    #[test]
    fn unlock_with_queued_requests() {
        let mut lock_thread = LockThread::new();
        let mut ftl_lock = FtlLock {
            lock: PTHREAD_MUTEX_INITIALIZER,
            ftl_waiting_for_lock: false
        };
        let (sender, receiver) = channel();
        let (queued_sender, queued_receiver) = channel();

        lock_thread.lock_count = 1;
        lock_thread.wait_queue.push_back(queued_sender);

        // Lock the mutex
        lock_mutex(&mut ftl_lock.lock);

        lock_thread.unlock(&mut ftl_lock, sender);

        // Lock count stayed at 1, because it was decremented and then incremented
        assert_eq!(lock_thread.lock_count, 1);

        // The mutex ended up still locked
        assert_eq!(unsafe { pthread_mutex_trylock(&mut ftl_lock.lock) }, EBUSY);
        unlock_mutex(&mut ftl_lock.lock);

        // A successful response has been sent to the unlock requester
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 0);

        // A successful response has been sent to the lock requester
        assert_eq!(queued_receiver.try_recv().unwrap().unwrap(), 0);

        destroy_lock(ftl_lock.lock);
    }
}
