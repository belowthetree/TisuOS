//! 字符工具库（内核用）
//! 提供数字字符转换功能
//! 2020年12月31日 zg

#![allow(dead_code)]

pub fn write_str(ptr : *mut char, s : &String, len : usize) {
    let bytes = s.as_bytes();
    for i in 0..len {
        if i < s.len() {
            unsafe {
                ptr.add(i).write_volatile(bytes[i] as char);
            }
        }
        else {
            unsafe {
                ptr.add(i).write_volatile('\0' as char);
                break;
            }
        }
    }
}

pub fn from_ptr(ptr : *const char)->String {
    let mut rt = String::new();
    let mut idx = 0;
    while unsafe {*(ptr.add(idx))} != '\0' {
        rt.push(unsafe {*(ptr.add(idx))});
        idx += 1;
    }
    rt
}

pub fn to_char_slice(s : &String)->Vec<char> {
    let mut v = Vec::new();
    for c in s.bytes() {
        v.push(c as char);
    }
    v
}

pub fn usize_to_str(num : usize)->String {
    let mut rt = String::new();
    let mut num = num;
    while num != 0 {
        let t = num % 10;
        rt.push(t as u8 as char);
        num = num / 10;
    }
    rt
}

pub fn from_u64(num : u64) ->String {
    let mut n = num;
    let mut rt = String::new();
    for _ in 0..8{
        rt.push((n & 0xff) as u8 as char);
        n = n >> 8;
    }
    rt
}

pub fn from_u16(num : u16)->String{
    let mut n = num;
    let mut rt = String::new();
    for _ in 0..2{
        rt.push((n & 0xff) as u8 as char);
        n = n >> 8;
    }
    rt
}

pub fn as_u64(s : String)->u64{
    let mut rt : u64 = 0;
    for (i, c) in s.bytes().enumerate(){
        if i >= 8{
            break;
        }
        rt = (rt >> 8) | ((c as u64 )<< 56);
    }
    if s.len() < 8{
        rt = rt >> ((8 - s.len()) * 8);
    }
    rt
}

pub fn convert_to_usize(s : &String)->usize{
    let mut rt : usize = 0;
    for c in s.bytes(){
        rt = rt * 10 + c as usize - '0' as usize;
    }
    rt
}

/// ## 短文件名校验和计算
pub fn check_sum(s : &String)->u8{
    let mut rt : u8 = 0;
    for (i, c) in s.bytes().enumerate(){
        if i >= 11 {
            break;
        }
        if rt & 1 == 1{
            rt = rt >> 1;
            rt = rt.wrapping_add(c).wrapping_add(0x80);
        }
        else {
            rt = (rt >> 1) + c;
        }
    }
    rt
}

/// ## 从后往前拆
pub fn split_back(s : &String, p : char)->(&str, &str){
    let mut idx = 0;
    for (i, c) in s.bytes().enumerate(){
        if c as char == p{
            idx = i;
        }
    }
    if idx != 0{
        s.split_at(idx)
    }
    else{
        (&s[..], "")
    }
}

/// ## 获取对应的短文件名
/// fat32 要求的短文件名，这里不检测重名递增
pub fn make_shortname(name : &String)->String{
    let tmp = name.clone().to_uppercase();
    let mut name = String::new();
    for c in tmp.split(' '){
        name += c;
    }
    let (head, tail) = split_back(&name, '.');
    let mut _s = String::new();
    if head.len() < 8{
        _s = head.to_string();
        while _s.len() < 8{
            _s += " ";
        }
    }
    else {
        _s = head.split_at(6).0.to_string();
        _s += "~1";
    }
    _s += tail;
    while _s.len() < 11{
        _s += " ";
    }
    _s
}

pub fn char_to_str(cs : &[char])->String {
    let mut rt = String::new();
    for c in cs {
        rt.push(*c);
    }
    rt
}


use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;