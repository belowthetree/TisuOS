use core::fmt::Write;
use alloc::vec::Vec;
use crate::libs::syscall::write;

pub struct Stdio;

impl Stdio {
    pub const fn new()->Self {
        Self
    }
}

impl Write for Stdio {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut v = Vec::new();
        for c in s.bytes() {
            v.push(c);
        }
        let data = v.as_slice();
        write(1, data);
        Ok(())
    }
}