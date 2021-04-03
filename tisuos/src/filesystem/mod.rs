//! # 文件系统
//! 
//! 2021年1月29日 zg

pub mod filetree;
pub mod elf;
pub mod image;
mod interface;
mod format;
mod block_info;
mod require;
mod implement;
use format::fat32;

#[allow(dead_code)]
pub enum Mgr {
    None,
    FAT(FATManger),
}

pub static mut MANAGER : Option<Vec<Mgr>> = None;

pub fn init(){
    unsafe {
        let mut infos = Vec::<Mgr>::new();
        if let Some(dev) = &mut DEVICES{
            for idx in 0..dev.block_device.len(){
                let info = BlockInfo::new(idx);
                match info.get_type() {
                    FAT32 => {
                        let info = FATInfo::new(&info.head);
                        let mgr = FATManger::new(&*info, idx);
                        print!("disk {} is fat, total size {}MB\t\t", idx, mgr.get_total_size() / 1024 / 1024);
                        println!("used size {}MB\t\t", mgr.get_used_size() / 1024 / 1024);
                        infos.push(Mgr::FAT(mgr));
                    }
                    _ => {
                        println!("unknown filesystem");
                        continue;
                    }
                }
            }
        }
        MANAGER = Some(infos);
        file::init();
        image::image::init();
    }
}


use crate::{uart, virtio::{device::DEVICES}};
use alloc::prelude::v1::*;
use block_info::BlockInfo;
use filetree::file;
use require::DoInfo;
use fat32::FATInfo;
use block_info::BlockType::{FAT32};
use self::{fat32::FATManger};