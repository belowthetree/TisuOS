use alloc::prelude::v1::*;
use fs_format::FileInfo;

use crate::libs::{Block, syscall::{file_info, free, open, read}};

pub struct File {}

impl File {
    pub fn read(path: String)->Result<Block<u8>, FileError> {
        let id = open(path.clone(), 1);
        if id < 0 {
            return Err(FileError::OpenFail);
        }
        let ptr = file_info(path);
        let info = unsafe {&*(ptr as *const FileInfo)};
        let data = Block::<u8>::new(info.size);
        let len = read(id as usize, data.to_array(0, info.size));
        free(ptr);
        if len <= 0 {
            return Err(FileError::ReadFail);
        }
        Ok(data)
    }
}

#[derive(Debug)]
pub enum FileError {
    OpenFail,
    ReadFail,
}