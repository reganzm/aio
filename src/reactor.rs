use std::{
    cell::RefCell,
    os::unix::prelude::RawFd,
    rc::Rc,
    task::{Context, Waker},
};

use polling::{Event, Poller};

use crate::excutor::EX;

/// get_reactor returns the reactor reference
#[inline]
pub(crate) fn get_reactor() -> Rc<RefCell<Reactor>> {
    EX.with(|ex| ex.reactor.clone())
}

#[derive(Debug)]
pub struct Reactor {
    poller: Poller,
    waker_mapping: rustc_hash::FxHashMap<u64, Waker>,
    buffer: Vec<Event>,
}

impl Reactor {
    pub fn new() -> Self {
        Self {
            poller: Poller::new().unwrap(),
            waker_mapping: Default::default(),
            buffer: Vec::with_capacity(2048),
        }
    }

    // Epoll related
    pub fn add(&mut self, fd: RawFd) {
        let flags =
            nix::fcntl::OFlag::from_bits(nix::fcntl::fcntl(fd, nix::fcntl::F_GETFL).unwrap())
                .unwrap();
        let flags_nonblocking = flags | nix::fcntl::OFlag::O_NONBLOCK;
        nix::fcntl::fcntl(fd, nix::fcntl::F_SETFL(flags_nonblocking)).unwrap();
        // add fd to poller and register no interest
        self.poller
            .add(fd, polling::Event::none(fd as usize))
            .unwrap();
    }

    pub fn modify_readable(&mut self, fd: RawFd, cx: &mut Context) {
        // record the token and waker mapping, the key = fd * 2
        self.push_completion(fd as u64 * 2, cx);
        let event = polling::Event::readable(fd as usize);
        // add the readable event to poller
        let _ = self.poller.modify(fd, event);
    }

    pub fn modify_writable(&mut self, fd: RawFd, cx: &mut Context) {
        // record the token and waker mapping, the key = fd * 2 + 1
        self.push_completion(fd as u64 * 2 + 1, cx);
        let event = polling::Event::writable(fd as usize);
        // and the wirtable event to poller
        let _ = self.poller.modify(fd, event);
    }

    pub fn wait(&mut self) {
        // waits for at least one event to be available
        let _ = self.poller.wait(&mut self.buffer, None);
        // process the events
        for _ in 0..self.buffer.len() {
            let event = self.buffer.swap_remove(0);
            if event.readable {
                if let Some(waker) = self.waker_mapping.remove(&(event.key as u64 * 2)) {
                    // the read event is ready, wake the waker
                    waker.wake();
                }
            }
            if event.writable {
                if let Some(waker) = self.waker_mapping.remove(&(event.key as u64 * 2 + 1)) {
                    // the write event is ready, wake the waker
                    waker.wake();
                }
            }
        }
    }

    pub fn delete(&mut self, fd: RawFd) {
        // remove the fd from poller
        self.waker_mapping.remove(&(fd as u64 * 2));
        self.waker_mapping.remove(&(fd as u64 * 2 + 1));
    }

    /// push_completion pushes the token and waker mapping into the waker_mapping
    fn push_completion(&mut self, token: u64, cx: &mut Context) {
        self.waker_mapping.insert(token, cx.waker().clone());
    }
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}
