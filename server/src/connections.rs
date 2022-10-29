use std::ops::{Deref, DerefMut};

const POLLFLAGS: i16 = libc::POLLIN | libc::POLLRDHUP;

pub struct Connections {
    inner: Vec<libc::pollfd>,
}
impl Deref for Connections {
    type Target = Vec<libc::pollfd>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Connections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Connections {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
    pub fn push_from_fd(&mut self, fd: i32) {
        self.inner.push(libc::pollfd {
            fd,
            events: POLLFLAGS,
            revents: 0,
        });
    }
}
