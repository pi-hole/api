// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Lock Handler Thread
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::memory_model::FtlLock;
use libc::{self, pthread_mutex_lock, pthread_mutex_unlock};
use shmem::{Map, Object};
use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration
};
use util::Error;

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
#[derive(Debug)]
pub enum RequestType {
    Lock,
    Unlock
}

/// The lock thread handler. This thread takes in lock requests and keeps track
/// of open read locks.
pub struct LockThread {
    lock_count: usize,
    wait_queue: VecDeque<Sender<LockResponse>>
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
    fn lock(&mut self, shm_lock: &mut FtlLock, sender: Sender<LockResponse>) {
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
    fn unlock(&mut self, shm_lock: &mut FtlLock, sender: Sender<LockResponse>) {
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
