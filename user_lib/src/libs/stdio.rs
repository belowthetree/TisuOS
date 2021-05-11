use alloc::prelude::v1::*;
use crate::libs::syscall;

pub fn get_input()->Option<String> {
    let data = &mut [0;1024];
    let len = syscall::read(0, data);
    if len > 0 {
        let mut s = String::new();
        for i in 0..len as usize {
            s.push(data[i] as char);
        }
        Some(s)
    }
    else {
        None
    }
}