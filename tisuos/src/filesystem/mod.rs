//! # 文件系统
//! 
//! 2021年1月29日 zg

pub mod elf;
mod format;
mod disk_info;

use format::fat32;
use tisu_fs::{FileSystem, Format};

#[allow(dead_code)]
pub enum Mgr {
    None,
    FAT(FATManger),
}

pub static mut MANAGER : Option<Vec<Mgr>> = None;
pub static mut SYSTEM : Option<Vec<FileSystem>> = None;

pub fn init(){
    unsafe {
        let mut infos = Vec::<Mgr>::new();
        let mut sys = Vec::new();
        for idx in 0..get_device().block_device.len(){
            let info = DiskInfo::new(idx);
            match info.get_type() {
                FAT32 => {
                    let info = FATInfo::new(&info.head);
                    let mgr = FATManger::new(&*info, idx);
                    let system = mgr.to_system();
                    sys.push(system);
                    println!("disk {} is fat32, total size {}MB\t\t", idx, mgr.get_total_size() / 1024 / 1024);
                    // println!("used size {}MB\t\t", mgr.get_used_size() / 1024 / 1024);
                    infos.push(Mgr::FAT(mgr));
                }
                disk_info::BlockType::TianMu => {
                    let tm = TianMu::new(idx);
                    let system = tm.to_system();
                    sys.push(system);
                    println!("disk {} is tianmu, total size {}MB",
                        idx, tm.0.total_size / 1024 / 1024);
                }
                _ => {
                    println!("unknown filesystem");
                    continue;
                }
            }
        }
        SYSTEM = Some(sys);
        MANAGER = Some(infos);
    }
}

pub fn get_system(idx : usize)->Option<&'static mut FileSystem> {
    unsafe {
        if let Some(sys) = &mut SYSTEM {
            sys.get_mut(idx)
        }
        else {
            None
        }
    }
}

use crate::virtio::device::get_device;
use alloc::prelude::v1::*;
use disk_info::DiskInfo;
use fat32::FATInfo;
use disk_info::BlockType::FAT32;
use self::{fat32::FATManger, format::tianmu::TianMu};