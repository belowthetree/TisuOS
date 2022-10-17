//! # 各类文件系统格式
//! 
//! 2021年1月31日 zg

use alloc::{sync::Arc};
use tisu_fs::{DiskInfo, FileSystem, Leaf, LeafType};
use crate::virtio::disk_cache::get_cache;
use self::fat32::{Attribute, FATManger};
use alloc::vec::Vec;
use super::syscall_io::get_id_mgr;
use alloc::string::String;


pub mod fat32;
pub mod tianmu;
pub mod elf;
mod disk_type;

pub use disk_type::*;


impl FATManger {
    pub fn parse_dir(&mut self, dir_idx : usize)->Option<Vec<Leaf>> {
        let mut rt = Vec::new();
        let items = self.get_dir_items(dir_idx);
        if items.is_none() {
            return None;
        }
        let items = items.unwrap();
        let mut filename = String::new();
        for item in items{
            if item.attr == Attribute::LongName.val(){
                filename = item.filename + &filename[..];
            }
            else if item.attr != Attribute::LongName.val() {
                if filename.len() <= 0{
                    filename = item.filename.clone();
                }
                let ltype = if item.is_dir() {
                    LeafType::Directory
                } else { LeafType::File };
                rt.push(
                    Leaf{
                        name: filename.clone(),
                        ltype,
                        block_idx: item.start_cluster,
                        size: item.size,
                    }
                );
                filename.clear();
            }
            else{
                filename.clear();
            }
        }
        Some(rt)
    }

    pub fn to_system(mgr : Arc<Self>)->tisu_fs::FileSystem {
        FileSystem::new(get_cache(), mgr.clone(), get_id_mgr(), mgr.block_idx)
    }
}

impl tisu_fs::Format for fat32::FATManger {
    fn parse_node(&self, block_idx : usize)->Result<Vec<Leaf>, ()> {
        let mut rt = Vec::new();
        unsafe {
            let t = self as *const Self as *mut Self;
            let t = (*t).parse_dir(block_idx).unwrap();
            for mut item in t {
                if item.block_idx == 0 {
                    item.block_idx = 2;
                }
                rt.push(item);
            }
            Ok(rt)
        }
    }

    fn get_block_chain(&self, start_idx : usize)->Result<Vec<usize>, ()> {
        unsafe {
            let t = self as *const Self as *mut Self;
            Ok((*t).get_all_cluster(start_idx).unwrap())
        }
    }

    fn parse_super_block(&self)->DiskInfo {
        DiskInfo {
            stype: tisu_fs::SystemType::FAT32,
            total_size: 512 * 1024 * 1024,
            block_size: self.cluster_size,
            root_directory_block_idx: 2,
            block_start_addr: self.cluster_start_addr,
        }
    }

    fn get_device(&self) ->usize {
        self.block_idx
    }
}