//! # 文件
//! 文件结构以及存储方式,提供两种存储方式
//! * 链表式存储
//! * 连续大文件存储
//! 
//! 2021年1月1日 zg


/// ## 全局文件信息记录
/// 所有被打开的文件会被记录在这
pub static mut OPENED_FILES : Option<Vec<File>> = None;
pub static mut FILE_LOCK : Mutex = Mutex::new();

fn get_file_record(start_cluster : usize, block_idx : usize)->Option<File>{
    unsafe {
        FILE_LOCK.lock();
        if let Some(files) = &mut OPENED_FILES{
            for file in files.iter(){
                if file.start_cluster == start_cluster && file.block_idx == block_idx{
                    FILE_LOCK.unlock();
                    return Some(file.clone());
                }
            }
        }
        FILE_LOCK.unlock();
        None
    }
}
/// ## 更新全局文件信息
/// 如果不存在，则加入
fn update_file_record(file : &File){
    unsafe {
        FILE_LOCK.lock();
        if let Some(files) = &mut OPENED_FILES{
            for (idx, f) in files.iter().enumerate(){
                if f.start_cluster == file.start_cluster && f.block_idx == file.block_idx{
                    files[idx] = file.clone();
                    FILE_LOCK.unlock();
                    return;
                }
            }
            files.push(file.clone());
        }
        FILE_LOCK.unlock();
    }
}

fn open_file(file : &File, flag : u8)->Result<(), ()>{
    unsafe {
        FILE_LOCK.lock();
        if let Some(files) = &mut OPENED_FILES{
            for f in files{
                if f.start_cluster == file.start_cluster && f.block_idx == file.block_idx{
                    if f.is_write() || f.is_read() && flag & OpenFlag::Write.val() != 0{
                        FILE_LOCK.unlock();
                        return Err(());
                    }
                    f.open_cnt += 1;
                    FILE_LOCK.unlock();
                    return Ok(());
                }
            }
        }
        FILE_LOCK.unlock();
        Err(())
    }
}

fn close_file(file : &File){
    unsafe {
        FILE_LOCK.lock();
        if let Some(files) = &mut OPENED_FILES{
            for f in files{
                if f.start_cluster == file.start_cluster && f.block_idx == file.block_idx{
                    f.open_cnt -= 1;
                    FILE_LOCK.unlock();
                    return;
                }
            }
        }
        FILE_LOCK.unlock();
    }
}

/// ## 文件链
/// 磁盘中的文件可以选择以文件链的形式储存在内存中，可以更加高效地利用内存
/// 但是速度捉鸡，因为一个簇实在是太小了一个单位链以一个簇的大小为基本单位
/// 仅当尝试获取超出本簇范围的数据时才会新建下一个单位
// pub struct FileChain{
//     start_cluster : usize,
//     next_cluster : usize,
//     pub content : Box<Block>,
//     next : Option<Box<FileChain>>,
//     block_idx : usize,
// }
// impl FileChain {
//     pub fn new(start_cluster : usize, block_idx : usize) ->Option<Box<Self>> {
//         if let Some(mgr) = get_next_cluster(block_idx){
//             if let Some(b) = mgr.get_cluster(start_cluster){
//                 if let Some(item) = mgr.get_fat_item(start_cluster){
//                     Some(Box::new(
//                         Self{
//                             start_cluster : start_cluster,
//                             next_cluster : item.get_next_item_num() as usize,
//                             content : b,
//                             next : None,
//                             block_idx : block_idx
//                         }
//                     ))
//                 }
//                 else{
//                     None
//                 }
//             }
//             else{
//                 println!("no b");
//                 None
//             }
//         }
//         else{
//             None
//         }
//     }
//     pub fn get(&mut self, idx : usize)->Option<u8>{
//         if idx < self.content.size{
//             self.content.get(idx)
//         }
//         else if self.has_next(){
//             let idx = idx - self.content.size;
//             self.gen_next();
//             if let Some(next) = &mut self.next{
//                 next.get(idx)
//             }
//             else{
//                 None
//             }
//         }
//         else{
//             None
//         }
//     }
//     fn gen_next(&mut self) {
//         if let Some(_next) = &self.next{
//         } else{
//             self.next = FileChain::new(self.next_cluster, self.block_idx);
//         }
//     }
//     fn has_next(&self) ->bool {
//         self.next_cluster != 0
//     }

// }

/// ## 文件
/// 文件结构体作为文件的唯一合理存在标志
pub struct File{
    pub filename : String,
    pub start_cluster : usize,
    pub block_idx : usize,
    pub open_cnt : usize,
    pub size : usize,
    pub flag : u8,
}

impl Drop for File {
    fn drop(&mut self) {
        self.close();
    }
}
impl Clone for File{
    fn clone(&self) -> Self {
        Self{
            filename : self.filename[..].to_string(),
            start_cluster : self.start_cluster,
            block_idx : self.block_idx,
            open_cnt : self.open_cnt,
            size : self.size,
            flag : self.flag,
        }
    }
}
#[allow(dead_code)]
impl File {
    /// ## 创建新文件信息
    /// 只能通过文件树的项创建
    pub fn new(tree_item : &TreeItem)->Option<Self>{
        if !tree_item.is_file(){
            return None;
        }
        if let Some(file) = get_file_record(tree_item.start_cluster, tree_item.block_idx){
            return Some(file);
        }
        else{
            let file = File{
                filename : tree_item.filename[..].to_string(),
                start_cluster : tree_item.start_cluster,
                block_idx : tree_item.block_idx,
                size : tree_item.size,
                flag : 0,
                open_cnt : 0,
            };
            update_file_record(&file);
            Some(file)
        }
    }
    /// ## 打开文件
    /// 同一个文件只能被打开一次
    pub fn open(&mut self, flag : u8)->Result<(), ()>{
        if self.is_close() && open_file(self, flag).is_ok(){
            self.open_cnt += 1;
            self.flag = flag;
            Ok(())
        }
        else{
            Err(())
        }
    }
    /// ## 关闭文件
    /// 文件使用计数减一
    pub fn close(&mut self){
        if self.is_close(){
            return;
        }
        close_file(self);
        self.flag = OpenFlag::Close.val();
    }
    pub fn read(&self, idx : usize, len : usize)->Option<Block>{
        if idx >= self.size || self.is_close(){
            None
        }
        else {
            println!("file read cluster {}, idx {:x}, len {}", self.start_cluster, idx, len);
            read_content(self.block_idx, self.start_cluster, idx, len)
        }
    }
    pub fn is_open(&self)->bool{
        !self.is_close()
    }
    pub fn is_close(&self)->bool{
        self.flag == OpenFlag::Close.val()
    }
    pub fn is_write(&self)->bool{
        self.flag & OpenFlag::Write.val() != 0
    }
    pub fn is_read(&self)->bool{
        self.flag & OpenFlag::Read.val() != 0
    }
    
}


#[repr(u8)]
pub enum OpenFlag{
    Close = 0,
    Read = 1,
    Write = 2,
}
impl OpenFlag{
    pub fn val(self)->u8{
        self as u8
    }
}


use crate::{memory::block::{Block}, sync::Mutex, uart};
use super::{operation::read_content, file_tree::TreeItem};
use alloc::{prelude::v1::*};

