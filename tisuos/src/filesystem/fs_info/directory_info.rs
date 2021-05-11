#![allow(dead_code)]

use core::mem::size_of;
// use alloc::prelude::v1::*;

#[derive(Debug)]
pub struct DirectoryInfo{
    pub block_idx : usize,
    pub device_id : usize,
    pub dir_num : usize,
    pub file_num : usize,
    pub name_len : usize,
    pub struct_size : usize,
}

impl DirectoryInfo {
    pub fn replace(
        &mut self, dir: &tisu_fs::Directory, dir_num: usize, file_num: usize, name_len : usize) {
        let t = size_of::<char>();
        let struct_size = dir_num * 15 * t + file_num * 15 * t + size_of::<Self>();
        let t = Self {
            block_idx: dir.block_idx,
            device_id: dir.device_id,
            dir_num,
            file_num,
            name_len,
            struct_size,
        };
        *self = t;
    }
}

