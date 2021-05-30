//! # 处理系统调用 IO
//!
//! 2021年5月3日 zg

use tisu_fs::{IdManager, SystemOp};

use crate::task::get_task_mgr;
use super::{ID_MANAGER, io_info::IoError, pop_task_in, push_output, push_task_in, search_system};


pub fn get_id_mgr()->&'static mut IdManager {
    let rt;
    unsafe {
        if let Some(mgr) = &mut ID_MANAGER {
            rt = Some(mgr)
        }
        else {
            rt = None
        }
        rt.unwrap()
    }
}

type IoResult = Result<usize, IoError>;

/// 读写接口为系统调用服务
pub fn read(program_id:usize, file_id : usize, data : &mut [u8])->IoResult {
    if file_id <= 1 {
        let mut len = 0;
        if file_id == 0 {
            for i in 0..data.len() {
                if let Some(c) = pop_task_in(program_id) {
                    data[i] = c as u8;
                }
                else {
                    break;
                }
                len = i + 1;
            }
        }
        else {
            let mgr = get_task_mgr().unwrap();
            len = mgr.get_stdout(program_id, data);
        }
        Ok(len)
    }
    else {
        if let Some(sys) = search_system(file_id) {
            let file = sys.file(file_id).unwrap();
            if !file.is_own(program_id) {
                return Err(IoError::NotOpen);
            }
            if let Ok(len) = sys.read(file_id, data) {
                Ok(len)
            }
            else {
                Err(IoError::ReadError)
            }
        }
        else {
            Err(IoError::FileIdError(file_id))
        }
    }
}

pub fn write(program_id:usize, file_id : usize, data : &[u8])->IoResult {
    if file_id <= 1 {
        // println!("write {} len {}", id, data.len());
        if file_id == 0 {
            for i in 0..data.len() {
                push_task_in(program_id, data[i] as char);
            }
        }
        else {
            let mgr = get_task_mgr().unwrap();
            mgr.stdout(program_id, data);
            for c in data {
                push_output(*c as char);
            }
        }
        Ok(data.len())
    }
    else {
        if let Some(sys) = search_system(file_id) {
            let file = sys.file(file_id).unwrap();
            if !file.is_own(program_id) {
                return Err(IoError::NotOpen);
            }
            if let Ok(len) = sys.write(file_id, data) {
                Ok(len)
            }
            else {
                Err(IoError::ReadError)
            }
        }
        else {
            Err(IoError::FileIdError(file_id))
        }
    }
}

