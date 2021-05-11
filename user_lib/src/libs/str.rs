#![allow(dead_code)]
use alloc::prelude::v1::*;

pub fn to_char_slice(s : &String)->Vec<char> {
    let mut v = Vec::new();
    for c in s.bytes() {
        v.push(c as char);
    }
    v
}

pub fn convert_to_usize(s : &String)->usize{
    let mut rt : usize = 0;
    for c in s.bytes(){
        rt = rt * 10 + c as usize - '0' as usize;
    }
    rt
}
