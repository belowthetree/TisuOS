use alloc::prelude::v1::*;
use fs_format::DirectoryInfo;

use crate::libs::syscall::{directory_info, free};

pub struct Directory {
    pub path : String,
    pub block_idx : usize,
    pub device_id : usize,
    pub file : Vec<String>,
    pub directory : Vec<String>,
}

impl Directory {
    pub fn read(path : String)->Self {
        let ptr = directory_info(path.clone());
        let info = unsafe {&*(ptr as *const DirectoryInfo)};
        let mut file = Vec::new();
        let mut directory = Vec::new();
        for i in 0..info.file_num {
            file.push(info.get_name(i));
        }
        for i in info.file_num..(info.dir_num + info.file_num) {
            directory.push(info.get_name(i));
        }
        free(ptr);
        Self {
            path,
            block_idx:info.block_idx,
            device_id:info.device_id,
            file,
            directory,
        }
    }
}