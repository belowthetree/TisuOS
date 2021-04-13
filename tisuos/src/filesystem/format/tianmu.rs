//! # 天目文件系统格式
//! 
//! 2021年4月13日 zg

#![allow(dead_code)]

pub const MAGIC : [u8;4] = [2,3,3,3];

#[derive(Clone, Copy)]
pub struct TianMu {
    jump1   : u8,
    jump2   : u8,
    jump3   : u8,
    oem     : [u8;8],
    bpb     : Setting,
}


#[derive(Clone, Copy)]
pub struct Setting {
    pub bytes_per_node : u64,
    pub node_num : u64,
    pub record_offset : u64,
    pub root_table_offset : u64,
}

#[repr(C)]
pub struct NodeMap {
    pub flag : u64,
}

#[repr(u8)]
pub enum Attribute {
    Free = 0,
    File = 1,
    Directory = 1 << 1,
    LongNameStart = 1 << 2,
    LongNameNext = 1 << 3,
    LongNameEnd = 1 << 4,
}

#[repr(C)]
pub enum Item {
    LongName(LongName),
    ShortName(ShortName),
}

#[repr(C)] // 32 字节
pub struct Table {
    pub attr : Attribute,
    pub items : &'static mut [Item],
}

#[repr(C)] // 31 字节
pub struct LongName {
    pub name : [u8;29],
    pub next : u16,
}

#[repr(C)] // 31 字节
pub struct ShortName {
    pub name : [u8;8],
    pub create_time : [u8;3], // 时、分、秒
    pub create_year : u16,    // 最大 65535 年
    pub create_date : [u8;2], // 月、日
    pub start_node : u64,
    pub length : u64,
}
