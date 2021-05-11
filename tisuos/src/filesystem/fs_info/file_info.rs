//! # 文件信息
//!
//! 2021年4月29日 zg

#[derive(Debug)]
pub struct FileInfo {
    pub id : usize,
    pub device_id : usize,
    pub start_idx : usize,
    pub flag : usize,
    pub size : usize,
}

impl Clone for FileInfo {
    fn clone(&self) -> Self {
        Self {
            id:self.id,
            device_id:self.device_id,
            start_idx:self.start_idx,
            flag:self.flag,
            size:self.size,
        }
    }
}

impl FileInfo {
    pub fn new(
        id:usize,
        device_id:usize,
        start_idx:usize,
        flag:usize,
        size:usize,
    )->Self {
        Self {
            id,
            device_id,
            start_idx,
            flag,
            size,
        }
    }

}
