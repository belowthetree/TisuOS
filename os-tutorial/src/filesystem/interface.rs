//! # 文件操纵接口
//! 提供底层无关的文件操作接口
//! 
//! 2021年1月19日 zg

pub fn open(path : &String)->Option<File>{
    let s : Vec<&str> = path.split('/').collect();
    if s.len() <= 1 {
        return None;
    }
    
    let block_idx = s.first().unwrap();
    let block_idx = convert_to_usize(&block_idx.to_string());
    if let Some(root) = get_root(block_idx as usize){
        let mut tree = root;
        for (idx, dir) in s.iter().enumerate() {
            if idx == 0 || idx == s.len() - 1{
                continue;
            }
            if let Some(t) = tree.get_sub_directory(&dir.to_string()) {
                tree = t;
            }
            else {
                return None;
            }
        }
        tree.get_file(s.last().unwrap().to_string())
    }
    else{
        None
    }
}


use alloc::{prelude::v1::*};
use crate::{libs::str::convert_to_usize};

use super::{file::File, operation::get_root};
