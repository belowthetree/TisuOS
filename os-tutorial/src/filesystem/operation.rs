#![allow(dead_code)]
//! # 中继操作
//! 提供根目录、磁盘信息获取
//! 是上层文件抽象与底层文件系统的中继
//! 
//! 2020年12月18日 zg

use alloc::{prelude::v1::*};

pub static mut FATMANAGER : Option<Vec<fat32::FATManger>> = None;

///
/// 功能接口
/// 

pub fn init(){
    unsafe {
        let mut infos = Vec::<FATManger>::new();
        if let Some(blocks) = &mut BLOCKS{
            for idx in 0..blocks.len(){
                let info = get_fat_info(idx).addr as *const FATInfo;
                // 默认是 fat
                // if !(*info).is_fat(){
                //     println!("continue");
                //     continue;
                // }
                let mut mgr = FATManger::new(&*info, idx);
                print!("disk {} total size {}MB\t\t", idx, mgr.get_total_size() / 1024 / 1024);
                println!("used size {}MB\t\t", mgr.get_used_size() / 1024 / 1024);
                infos.push(mgr);
            }
        }
        FATMANAGER = Some(infos);
        OPENED_FILES = Some(Vec::<File>::new());
    }
    
}
/// ## 读取磁盘内容
/// 封装了底层文件系统的操作
pub fn read_content(block_idx : usize, cluster : usize, offset : usize, len : usize)->Option<Block>{
    if let Some(mgr) = get_mgr(block_idx){
        unsafe {
            buffer::DEBUG = true;
        }
        mgr.read(block_idx, cluster, offset, len)
    }
    else{
        None
    }
}
/// ## 写入磁盘内容
pub fn write_content(block_idx : usize, cluster : usize, offset : usize, len : usize, content : &Box<Block>){
    if let Some(mgr) = get_mgr(block_idx){
        mgr.write(block_idx, cluster, offset, len, content)
    }
}

/// ## 获取目录的文件树
/// @ block_idx : 磁盘号
/// @ cluster : 传入目录的簇号
pub fn get_directory(block_idx : usize, cluster : usize) ->Option<FileTree> {
    if let Some(mgr) = get_mgr(block_idx){
        mgr.get_file_tree(cluster)
    }
    else{
        None
    }
}
/// ## 获取磁盘根目录
/// 默认使用 fat32
pub fn get_root(block_idx : usize) ->Option<FileTree>{
    if let Some(mgr) = get_mgr(block_idx){
        mgr.get_root_file_tree()
    }
    else{
        None
    }
}

pub fn delete_file(tree : &FileTree, name : &String){
    if let Some(mgr) = get_mgr(tree.block_idx){
        mgr.delete_file(tree, name);
    }
}

pub fn create_directory(tree : &FileTree, name : &String){
    if let Some(mgr) = get_mgr(tree.block_idx){
        mgr.create_directory(tree, name);
    }
}

fn get_mgr<'a>(block_idx : usize) ->Option<&'a mut FATManger>{
    unsafe {
        if let Some(mgrs) = &mut FATMANAGER{
            // println!("mgr len {} {}", mgrs.len(), block_idx);
            if block_idx < mgrs.len(){
                Some(&mut mgrs[block_idx])
            }
            else{
                None
            }
        }
        else{
            None
        }
    }
}

/// ## 获取磁盘的 fat32 信息
fn get_fat_info(block_idx : usize) ->Block{
    let block = Block::new(512);
    sync_read_buffer(block_idx, block.addr, 512, 0);
    block
}

pub fn test(){
    // if let Some(mgr) = get_mgr(0){
    // }
}

///
/// ## 接口要求
/// 

pub trait IO {
    fn read(&mut self, block_idx : usize, cluster : usize, offset : usize, len : usize)->Option<Block>;
    fn write(&mut self, block_idx : usize, cluster : usize, offset : usize, len : usize, content : &Box<Block>);
}

pub trait Directory {
    fn get_file_tree(&mut self, cluster : usize)->Option<FileTree>;
    fn get_root_file_tree(&mut self)->Option<FileTree>;
    fn create_directory(&mut self, tree : &FileTree, name : &String)->bool;
    fn clear_directory(&mut self, cluster : usize);
    fn delete_file(&mut self, tree : &FileTree, name : &String)->bool;
}

pub trait BlockInfo {
    fn get_total_size(&self)->usize;
    fn get_used_size(&mut self)->usize;
}

use crate::{uart, virtio::buffer::{self, sync_read_buffer}};
use super::{fat32, file::{File, OPENED_FILES}, file_tree::{FileTree}};
use crate::virtio::block_device::{BLOCKS};
use crate::memory::block::{Block};
use super::fat32::{FATManger};
use fat32::{FATInfo};
