//! # 接口功能
//! 上层的虚拟文件系统通过接口完成各种功能
//! 
//! 2021年1月30日 zg

pub fn read_content(block_idx : usize, start_cluster : usize, idx : usize, len : usize)->Option<Block<u8>> {
    if let Some(mgr) = get_mgr(block_idx){
        match mgr {
            Mgr::FAT(fat) => {
                fat.read(block_idx, start_cluster, idx, len)
            }
            _ => {None}
        }
    }
    else{
        None
    }
}

pub fn write_content(block_idx : usize, start_cluster : usize, idx : usize, len : usize, ctx : &Block<u8>) {
    if let Some(mgr) = get_mgr(block_idx){
        match mgr {
            Mgr::FAT(fat) => {
                fat.write(block_idx, start_cluster, idx, len, ctx);
            }
            _ => {}
        }
    }
}

pub fn get_mgr<'a>(block_idx : usize) ->Option<&'a mut Mgr>{
    unsafe {
        if let Some(mgrs) = &mut MANAGER{
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


use crate::memory::block::Block;

use super::{MANAGER, Mgr, require::DoIO};

