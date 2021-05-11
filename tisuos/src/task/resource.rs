//! # 任务资源
//! 管理任务对公共资源的占用、释放
//!
//! 2021年5月4日 zg


use alloc::prelude::v1::*;
use tisu_fs::SystemOp;
use crate::filesystem::search_system;

pub struct Resource {
    pid : usize,
    files : Vec<usize>,
}

impl Resource {
    pub fn new(pid : usize)->Self {
        Self {
            pid,
            files : Vec::new(),
        }
    }

    pub fn push_file(&mut self, id:usize) {
        self.files.push(id);
    }

    pub fn release_file(&mut self, id:usize) {
        for (i, file_id) in self.files.iter().enumerate() {
            if id == *file_id {
                self.files.remove(i);
                break;
            }
        }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        for file_id in self.files.iter() {
            if let Some(sys) = search_system(*file_id) {
                let file = sys.file(*file_id).unwrap();
                for (i, owner) in file.state.owner.iter().enumerate() {
                    if *owner == self.pid {
                        file.state.owner.remove(i);
                        break;
                    }
                }
            }
            else {
                panic!("no file id {}", file_id);
            }
        }
    }
}