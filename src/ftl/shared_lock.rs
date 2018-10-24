// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Lock
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use failure::{Fail, ResultExt};
use ftl::lock_thread::{LockRequest, LockThread, RequestType};
use nix::errno::Errno;
use std::{
    sync::{
        mpsc::{channel, Sender},
        Mutex
    },
    thread
};
use util::{Error, ErrorKind};

/// A lock for coordinating shared memory access with FTL. It locks a mutex in
/// shared memory, and while holding the lock it distributes read locks. If it
/// detects that FTL is waiting for a lock on the shared mutex, it will stop
/// distributing read locks until FTL gets the lock back.
///
/// The shared memory lock must be locked and unlocked from the same thread, so
/// the locking happens on a dedicated lock handling thread.
pub struct ShmLock {
    sender: Mutex<Sender<LockRequest>>
}

impl ShmLock {
    /// Create a new `ShmLock` with a lock count of zero.
    pub fn new() -> ShmLock {
        // Create a lock thread which handles taking the shared lock, since
        // pthread doesn't like locking and unlocking from different threads.
        let (request_sender, request_receiver) = channel();

        thread::Builder::new()
            .name("Lock Handler".to_owned())
            .spawn(move || {
                let mut lock_thread = LockThread::new();
                lock_thread.handle_requests(request_receiver);
            })
            .unwrap();

        ShmLock {
            sender: Mutex::new(request_sender)
        }
    }

    /// Acquire a read lock on the shared memory. It will last as long as the
    /// guard (return value) lives.
    pub fn read(&self) -> Result<ShmLockGuard, Error> {
        self.send_request(RequestType::Lock)?;
        Ok(ShmLockGuard::Production { lock: self })
    }

    /// Send a request to the lock thread. This will block until the request
    /// has finished. For a lock request, this is until the lock is obtained.
    /// For an unlock request, this is until the lock has been unlocked.
    pub(self) fn send_request(&self, request: RequestType) -> Result<(), Error> {
        let (sender, receiver) = channel();

        // Lock access to the lock thread. Ignore the poison error because the
        // state of the sender should still be consistent.
        let lock_thread = self.sender.lock().unwrap_or_else(|e| e.into_inner());

        lock_thread
            .send((request, sender))
            .map_err(|_| Error::from(ErrorKind::SharedMemoryLock))?;

        // The lock thread guard is dropped so other threads can communicate
        // with the lock thread while this thread waits for a response.
        drop(lock_thread);

        let ret = receiver.recv().context(ErrorKind::SharedMemoryLock)??;

        if ret != 0 {
            Err(Error::from(
                Errno::from_i32(ret).context(ErrorKind::SharedMemoryLock)
            ))
        } else {
            Ok(())
        }
    }
}

/// A RAII type lock guard which keeps the lock active until it is dropped.
pub enum ShmLockGuard<'lock> {
    Production {
        lock: &'lock ShmLock
    },
    #[cfg(test)]
    Test
}

impl<'lock> Drop for ShmLockGuard<'lock> {
    fn drop(&mut self) {
        match self {
            ShmLockGuard::Production { lock } => {
                lock.send_request(RequestType::Unlock).unwrap();
            }
            #[cfg(test)]
            ShmLockGuard::Test => ()
        }
    }
}

#[cfg(test)]
mod test {
    use ftl::{
        lock_thread::{LockRequest, RequestType},
        ShmLock
    };
    use std::{
        sync::{
            mpsc::{channel, Receiver},
            Mutex
        },
        thread
    };

    /// Wait for a millisecond before trying to get a response and checking the
    /// request type. After asserting the request type is correct, send a
    /// successful response.
    fn check_request(receiver: &Receiver<LockRequest>, request_type: RequestType) {
        let (request, response_sender) = receiver.recv().unwrap();

        assert_eq!(request, request_type);

        response_sender.send(Ok(0)).unwrap();
    }

    /// Check that the correct requests are sent to the lock thread when
    /// locking and unlocking
    #[test]
    fn send_correct_requests() {
        // Initialize the ShmLock
        let (sender, receiver) = channel();

        let lock = ShmLock {
            sender: Mutex::new(sender)
        };

        // Create the mock lock handler thread
        let handler_thread = thread::spawn(move || {
            check_request(&receiver, RequestType::Lock);
            check_request(&receiver, RequestType::Unlock);
        });

        // Take a lock
        let guard = lock.read().unwrap();

        // Drop the lock (check unlock functionality)
        drop(guard);

        // Join with the mock lock handler thread
        handler_thread.join().unwrap();
    }

    /// Check that error returned from the pthread lock call (returned as
    /// Ok(error_num)) is returned as an `Error`.
    #[test]
    fn return_pthread_lock_errors() {
        // Initialize the ShmLock
        let (sender, receiver) = channel();

        let lock = ShmLock {
            sender: Mutex::new(sender)
        };

        // Create the mock lock handler thread
        let handler_thread = thread::spawn(move || {
            let (_, response_sender) = receiver.recv().unwrap();

            // Send an error (returned from pthread unlock call)
            response_sender.send(Ok(1)).unwrap();
        });

        // Take a lock (should be a Result::Err)
        let guard_result = lock.read();

        assert!(guard_result.is_err());

        // Join with the mock lock handler thread
        handler_thread.join().unwrap();
    }

    /// Check that error returned from the pthread unlock call (returned as
    /// Ok(error_num)) is returned as an `Error`.
    #[test]
    fn return_pthread_unlock_errors() {
        // Initialize the ShmLock
        let (sender, receiver) = channel();

        let lock = ShmLock {
            sender: Mutex::new(sender)
        };

        // Create the mock lock handler thread
        let handler_thread = thread::spawn(move || {
            let (_, response_sender) = receiver.recv().unwrap();

            // Send an error (returned from pthread unlock call)
            response_sender.send(Ok(1)).unwrap();
        });

        // Should be an error because the thread returned a return code of 1
        let unlock_result = lock.send_request(RequestType::Unlock);
        assert!(unlock_result.is_err());

        // Join with the mock lock handler thread
        handler_thread.join().unwrap();
    }
}
