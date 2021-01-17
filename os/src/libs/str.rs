//! 字符工具库（内核用）
//! 提供数字字符转换功能
//! 2020年12月31日 zg

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

pub fn to_u64(s : String)->u64{
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
    let mut s = String::new();
    if head.len() < 8{
        s = head.to_string();
        while s.len() < 8{
            s += " ";
        }
    }
    else {
        s = head.split_at(6).0.to_string();
        s += "~1";
    }
    s += tail;
    while s.len() < 11{
        s += " ";
    }
    s
}


use alloc::{prelude::v1::*};
