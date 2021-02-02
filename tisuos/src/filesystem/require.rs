//! # 文件系统接口要求
//! 所有操作要求体现在此,作为操作中继,实现文件系统格式无关
//! 
//! 2021年1月30日 zg


pub trait DoIO {
    fn read(&mut self, block_idx : usize, cluster : usize, offset : usize, len : usize)->Option<Block<u8>>;
    fn write(&mut self, block_idx : usize, cluster : usize, offset : usize, len : usize, content : &Block<u8>);
}

pub trait DoDirectory {
    fn get_directory(&mut self, cluster : usize)->Option<directory::Directory>;
    fn get_root_directory(&mut self)->Option<directory::Directory>;
    fn create_directory(&mut self, tree : &directory::Directory, name : &String)->bool;
    fn clear_directory(&mut self, cluster : usize);
    fn delete_file(&mut self, tree : &directory::Directory, name : &String)->bool;
}

pub trait DoInfo {
    fn get_total_size(&self)->usize;
    fn get_used_size(&self)->usize;
}


use crate::memory::block::Block;
use alloc::prelude::v1::*;
use super::filetree::directory;
