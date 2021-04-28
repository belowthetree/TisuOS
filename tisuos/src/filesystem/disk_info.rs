//! # 硬盘头部信息
//! 
//! 2021年1月29日 zg


pub struct DiskInfo {
    pub head : Block<u8>,
}

impl DiskInfo {
    pub fn new(block_idx : usize)->Self {
        let head = Block::new(1024);
        sync_read_buffer(block_idx, head.to_array(0, 1024), 0);
        Self {
            head,
        }
    }

    pub fn get_type(&self)->BlockType {
        let fat32 = [0x52, 0x52, 0x61, 0x41];
        let tianmu = [0x33,0x23,0,0];
        let mut flag = [0;4];
        for i in 512..516 {
            flag[i - 512] = self.head.get(i).unwrap();
        }
        if fat32 == flag {
            BlockType::FAT32
        }
        else if tianmu == flag {
            BlockType::TianMu
        }
        else {
            BlockType::Unknown
        }
    }
}

pub enum BlockType {
    Unknown,
    TianMu,
    FAT32,
}


use crate::{memory::block::Block, virtio::disk_cache::sync_read_buffer};
