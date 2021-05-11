//! # 硬盘头部信息
//! 
//! 2021年1月29日 zg


pub struct DiskType {
    device_id : usize,
    pub head : Block<u8>,
}

impl DiskType {
    pub fn new(device_id : usize)->Self {
        let head = Block::new(1024);
        sync_read_buffer(device_id, head.to_array(0, 1024), 0);
        Self {
            device_id,
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
            let info = FATInfo::new(&self.head);
            let mgr = FATManger::new(unsafe {&*info}, self.device_id);
            BlockType::FAT32(Arc::new(mgr))
        }
        else if tianmu == flag {
            let tm = TianMu::new(self.device_id);
            BlockType::TianMu(Arc::new(tm))
        }
        else {
            BlockType::Unknown
        }
    }
}

pub enum BlockType {
    Unknown,
    TianMu(Arc<TianMu>),
    FAT32(Arc<FATManger>),
}


use alloc::sync::Arc;

use crate::{memory::block::Block, virtio::disk_cache::sync_read_buffer};

use super::{fat32::{FATInfo, FATManger}, tianmu::TianMu};
