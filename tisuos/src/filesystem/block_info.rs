//! # 硬盘头部信息
//! 
//! 2021年1月29日 zg


pub struct BlockInfo {
    pub head : Block<u8>,
}

impl BlockInfo {
    pub fn new(block_idx : usize)->Self {
        let head = Block::new(512);
        sync_read_buffer(block_idx, &head, 0, 512, 0);
        Self {
            head : head,
        }
    }

    pub fn get_type(&self)->BlockType {
        let mut s = String::new();
        for i in 0..self.head.size {
            let c = self.head.get(i).unwrap() as char;
            s.push(c);
        }
        if s.contains("fat") {
            BlockType::FAT32
        }
        else {
            BlockType::Unknown
        }
    }
}

pub enum BlockType {
    Unknown,
    FAT32,
}


use crate::{memory::block::Block, virtio::buffer::sync_read_buffer};
use alloc::prelude::v1::*;
