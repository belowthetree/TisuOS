//! # 天目文件系统格式实现
//! 
//! 2021年4月13日 zg


use core::mem::size_of;

use tianmu_fs::{DirItem, SuperBlock};
use tisu_fs::{DiskInfo, FileSystem, Format, Leaf, LeafType};
use alloc::{prelude::v1::*, sync::Arc};
use crate::{filesystem::syscall_io::get_id_mgr, libs::bytes::{slice_to_string, slice_to_val}, memory::block::Block, virtio::disk_cache::get_cache};

#[derive(Debug, Clone, Copy)]
pub struct TianMu(pub fs_format::TianMu);

impl TianMu {
    pub fn new(device_id:usize)->Self {
        let cache = get_cache();
        let sp = Block::<SuperBlock>::new(1);
        cache.read(device_id, sp.to_array(0, size_of::<SuperBlock>()), 0);
        Self(fs_format::TianMu::new(device_id, sp.get(0).unwrap()))
    }

    pub fn to_system(tm : Arc<TianMu>)->FileSystem {
        FileSystem::new(
            get_cache(), tm.clone(), get_id_mgr(), tm.0.device_id)
    }

}

impl Format for TianMu {
    fn parse_node(&self, block_idx : usize)->Result<Vec<tisu_fs::Leaf>, ()> {
        let mut rt = Vec::new();
        let data = Block::<DirItem>::new(1);
        let cache = get_cache();
        let buf = data.to_array(0, size_of::<DirItem>());
        for idx in self.get_block_chain(block_idx).unwrap().iter() {
            let num = self.0.block_size / size_of::<DirItem>();
            let base = self.0.block_size * idx;
            let step = size_of::<DirItem>();
            for i in 0..num {
                let addr = base + i * step;
                cache.read(self.0.device_id, buf, addr);
                let item = data.get(0).unwrap();
                if item.empty() {
                    continue;
                }
                let ltype = if item.is_dir() {LeafType::Directory} else {LeafType::File};
                rt.push(Leaf {
                    name: slice_to_string(&item.name),
                    ltype,
                    block_idx: item.start_block as usize,
                    size: item.length as usize,
                });
            }
        }
        Ok(rt)
    }

    fn get_block_chain(&self, start_idx : usize)->Result<Vec<usize>, ()> {
        let cache = get_cache();
        let mut rt = Vec::new();
        let mut flag = start_idx as u64;
        let data = &mut [0;8];
        while flag != 0 && flag != tianmu_fs::END {
            let addr = flag as usize * 8 + self.0.block_map_addr;
            rt.push(flag as usize);
            cache.read(self.0.device_id, data, addr);
            flag = slice_to_val(data);
        }
        Ok(rt)
    }

    fn parse_super_block(&self)->DiskInfo {
        DiskInfo {
            stype: tisu_fs::SystemType::Tianmu,
            total_size: self.0.total_size,
            block_size: self.0.block_size,
            root_directory_block_idx: self.0.root_idx,
            block_start_addr: 0,
        }
    }

    fn get_device(&self)->usize {
        self.0.device_id
    }
}

// impl 
