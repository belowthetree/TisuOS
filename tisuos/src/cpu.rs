//! # 硬件相关指令
//!
//!
#![allow(dead_code)]

extern "C" {
    pub fn write_mscratch(v : usize);
    pub fn read_mhartid()->usize;
}


