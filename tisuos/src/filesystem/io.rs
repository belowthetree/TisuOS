//! # 内核用文件系统接口
//!
//! 2021年5月3日 zg

use alloc::prelude::v1::*;
use tisu_fs::{Directory, SystemOp};

use crate::{libs::str::convert_to_usize, memory::block::Block};

use super::get_system;

pub fn read(path:String)->Result<Block<u8>, ()> {
    let all_path = path;
    let idx = all_path.find("/").unwrap();
    let (id, path) = all_path.split_at(idx);
    let sys = get_system(convert_to_usize(&id.to_string())).unwrap();
    let file = sys.open(path.to_string(), tisu_fs::FileFlag::Read).unwrap().clone();
    let data = Block::<u8>::new(file.size);
    sys.read(file.id, data.to_array(0, file.size)).unwrap();
    Ok(data)
}

pub fn enter(path:String)->Result<Directory, ()> {
    let all_path = path;
    let idx = all_path.find("/").unwrap();
    let (id, path) = all_path.split_at(idx);
    let sys = get_system(convert_to_usize(&id.to_string())).unwrap();
    let dir = sys.enter(path.to_string()).unwrap();
    Ok(dir)
}