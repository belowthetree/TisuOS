//! # 硬盘头部信息
//! 
//! 2021年1月29日 zg


pub struct BlockInfo {
    pub head : Block<u8>,
}

impl BlockInfo {
    pub fn new(block_idx : usize)->Self {
        let head = Block::new(1024);
        sync_read_buffer(block_idx, &head, 0, 1024, 0);
        Self {
            head : head,
        }
    }

    pub fn get_type(&self)->BlockType {
        let fat32 = [0x52, 0x52, 0x61, 0x41];
        let mut flag = [0;4];
        for i in 512..516 {
            flag[i - 512] = self.head.get(i).unwrap();
        }
        if fat32 == flag {
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
