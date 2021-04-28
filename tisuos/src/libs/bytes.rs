#![allow(dead_code)]

use alloc::prelude::v1::*;

pub fn slice_to_string(s : &[u8])->String {
    let mut v = Vec::new();
    for c in s {
        if *c == 0 {
            break;
        }
        v.push(*c as u16);
    }
    String::from_utf16(v.as_slice()).unwrap()
}

pub fn slice_to_val<T:Copy>(s : &[u8])->T {
    unsafe {
        let t = s as *const [u8] as *const u8 as *const T;
        *t
    }
}

pub fn val_to_slice<T>(s : &mut[u8], v : T) {
    unsafe {
        let t = s as *mut [u8] as *mut u8 as *mut T;
        *t = v;
    }
}
