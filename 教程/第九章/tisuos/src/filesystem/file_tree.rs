//! # 文件树
//! 文件树只是对于目录结构的描述，其中的 Item 只是描述目录中的一个项，具体的文件操作应该由 File 执行
//! 
//! 2021年1月1日 zg

/// 文件树
/// 将磁盘文件、目录当作一个树形构造，一次提供一层（一个目录）的切片
/// 提供向上（前往父目录），向下（前往子目录）的功能，其它功能待补充
/// 父目录按照簇号存储进 Vec 中
pub struct FileTree{
    pub name : String,
    pub items : Vec::<TreeItem>,
    pub start_cluster : usize,
    pub parent_cluster : Vec<usize>,
    pub block_idx : usize,
}
impl FileTree {
    pub fn new(name : String, items : Vec::<TreeItem>, st_cluster : usize, block_idx : usize) ->Self {
        FileTree{
            name : name,
            items : items,
            start_cluster : st_cluster,
            parent_cluster : Vec::new(),
            block_idx : block_idx,
        }
    }
    #[allow(dead_code)]
    pub fn refresh(&mut self){
        let tree = get_directory(self.block_idx, self.start_cluster).unwrap();
        let parent = self.parent_cluster.clone();
        *self = tree;
        self.parent_cluster = parent;
    }
    /// 显示功能，待删除
    pub fn list_file(&self) {
        print!("files: ");
        for file in &self.items{
            if file.is_file(){
                print!("{} ", file.name);
            }
        }
        println!();
    }
    pub fn list_dir(&self) {
        print!("directories: ");
        for dir in &self.items{
            if dir.is_dir(){
                print!("{} {}", dir.name, dir.start_cluster);
            }
        }
        
        println!();
    }
    pub fn get_sub_directory(&self, name : &String) ->Option<FileTree>{
        for i in &self.items{
            if i.name == *name && i.is_dir(){
                if let Some(mut tree) = get_directory(self.block_idx, i.start_cluster){
                    for i in &self.parent_cluster{
                        tree.parent_cluster.push(*i);
                    }
                    tree.parent_cluster.push(self.start_cluster);
                    return Some(tree);
                }
                else{
                    println!("no dir");
                    return None;
                }
            }
        }
        None
    }
    pub fn get_parent_directory(&self) ->Option<FileTree>{
        if let Some(cluster) = self.parent_cluster.last(){
            if let Some(mut parent) = get_directory(self.block_idx, *cluster){
                let len = self.parent_cluster.len();
                for i in 0..len - 1{
                    parent.parent_cluster.push(self.parent_cluster[i]);
                }
                Some(parent)
            }
            else{
                None
            }
        }else{
            None
        }
    }
    /// 获取磁盘总大小
    pub fn get_total_size(&self) ->usize{
        unsafe {
            if let Some(mgr) = &FATMANAGER{
                mgr[self.block_idx].get_total_size()
            }
            else{
                0
            }
        }
    }
    /// 磁盘占用大小
    pub fn get_used_size(&self) ->usize{
        unsafe {
            if let Some(mgr) = &mut FATMANAGER{
                mgr[self.block_idx].get_used_size()
            }
            else{
                0
            }
        }
    }
    pub fn get_file(&self, filename : String) ->Option<File>{
        for item in &self.items{
            if item.is_file() && item.filename == filename{
                return item.get_file();
            }
        }
        None
    }
    /// 获取当前目录项的引用
    pub fn get_item<'a>(&'a self, filename : &String)->Option<&'a TreeItem> {
        for item in &self.items{
            if item.filename == *filename{
                return Some(item);
            }
        }
        None
    }
    pub fn has_shortname(&self, name : &String)->bool{
        for item in self.items.iter(){
            if make_shortname(&item.filename) == *name{
                return true;
            }
        }
        false
    }
}
pub struct TreeItem{
    pub filename : String,
    pub name : String,
    pub start_cluster : usize,
    pub block_idx : usize,
    pub size : usize,
    pub idx : usize,
    pub len : usize,
    attr : u8,
}
impl TreeItem {
    pub fn new(name : String, st_cluster : usize, attr : u8,
            block_idx : usize, size : usize, idx : usize, len : usize) -> Self {
        let mut s = String::new();
        for c in name.bytes(){
            if c as char != ' '{
                s.push(c as char);
            }
        }
        let filename = s.clone();
        TreeItem{
            filename : filename,
            name : s,
            start_cluster : st_cluster,
            block_idx : block_idx,
            size : size,
            idx : idx,
            len : len,
            attr : attr
        }
    }
    pub fn is_file(&self) ->bool {
        self.attr & Attribute::Archive.val() != 0
    }
    pub fn is_dir(&self) ->bool {
        self.attr & Attribute::SubDir.val() != 0
    }
    pub fn get_file(&self) ->Option<File>{
        if self.is_file(){
            File::new(self)
        }
        else{
            None
        }
    }
}

use alloc::{prelude::v1::*};
use crate::{libs::str::make_shortname, uart};
use super::{fat32::Attribute, file::{File}, operation::{FATMANAGER, get_directory, BlockInfo}};


