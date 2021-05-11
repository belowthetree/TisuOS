//! # 文件系统
//! 
//! 2021年1月29日 zg

mod format;
mod fs_info;
mod io_info;
mod stdio;
mod image_pool;
pub mod syscall_io;
pub mod io;

use tisu_fs::{FileSystem, IdManager, SystemOp};
pub use fs_info::*;
pub use stdio::*;
pub use format::elf;
pub use image_pool::request;

/// 一个文件系统对应一个磁盘
pub static mut SYSTEM : Option<Vec<FileSystem>> = None;
pub static mut FORMAT : Option<Vec<BlockType>> = None;
static mut ID_MANAGER : Option<IdManager> = None;

pub fn init(){
    stdio::init();
    unsafe {
        ID_MANAGER = Some(IdManager::new());
        let mut ftype = Vec::new();
        let mut sys = Vec::new();
        for idx in 0..get_device().block_device.len(){
            let info = DiskType::new(idx);
            match info.get_type() {
                BlockType::FAT32(mgr) => {
                    let system = FATManger::to_system(mgr.clone());
                    sys.push(system);
                    println!("disk {} is fat32, total size {}MB\t\t", idx, mgr.get_total_size() / 1024 / 1024);
                    // println!("used size {}MB\t\t", mgr.get_used_size() / 1024 / 1024);
                    ftype.push(BlockType::FAT32(mgr));
                }
                BlockType::TianMu(tm) => {
                    let system = TianMu::to_system(tm.clone());
                    sys.push(system);
                    println!("disk {} is tianmu, total size {}MB",
                        idx, tm.0.total_size / 1024 / 1024);
                    ftype.push(BlockType::TianMu(tm));
                }
                _ => {
                    println!("unknown filesystem");
                    continue;
                }
            }
        }
        FORMAT = Some(ftype);
        SYSTEM = Some(sys);
    }
}


pub fn get_system(idx : usize)->Option<&'static mut impl SystemOp> {
    unsafe {
        if let Some(sys) = &mut SYSTEM {
            sys.get_mut(idx)
        }
        else {
            None
        }
    }
}

pub fn search_system(id : usize)->Option<&'static mut impl SystemOp> {
    unsafe {
        if let Some(sys) = &mut SYSTEM {
            for sys in sys.iter_mut() {
                if sys.contain(id) {
                    return Some(sys);
                }
            }
            None
        }
        else {
            None
        }
    }
}

use crate::{filesystem::format::{fat32::FATManger, tianmu::TianMu}, virtio::device::get_device};
use alloc::prelude::v1::*;
use self::{format::{BlockType, DiskType}};