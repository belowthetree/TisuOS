//! # 目录
//! 
//! 2021年1月日 zg

/// ## 目录结构
/// 将磁盘文件、目录当作一个树形构造，一次提供一层（一个目录）的切片
/// 提供向上（前往父目录），向下（前往子目录）的功能，其它功能待补充
/// 父目录按照簇号存储进 Vec 中
pub struct Directory{
    pub name : String,
    pub items : Vec::<DirItem>,
    pub start_cluster : usize,
    pub parent_cluster : Vec<usize>,
    pub block_idx : usize,
}

// #[allow(dead_code)]
impl Directory {
    pub fn open(path : &String)->Option<Self> {
        let s : Vec<&str> = path.split('/').collect();
        
        let block_idx = s.first().unwrap();
        let block_idx = convert_to_usize(&block_idx.to_string());
        if let Some(root) = get_directory(block_idx as usize, 2) {
            let mut tree = root;
            for (idx, dir) in s.iter().enumerate() {
                if idx == 0 || dir.len() <= 0 {
                    continue;
                }
                if let Some(t) = tree.get_sub_directory(&dir.to_string()) {
                    tree = t;
                }
                else {
                    return None;
                }
            }
            Some(tree)
        }
        else{
            println!("no root");
            None
        }
    }
    pub fn new(name : String, items : Vec::<DirItem>, st_cluster : usize, block_idx : usize)->Self {
        Directory {
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
    
    pub fn get_sub_directory(&self, name : &String) ->Option<Directory>{
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

    pub fn get_parent_directory(&self) ->Option<Directory>{
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
    
    pub fn get_file(&self, filename : String) ->Option<File>{
        for item in &self.items{
            if item.is_file() && item.filename == filename{
                return item.get_file();
            }
        }
        None
    }
    /// 获取当前目录项的引用
    pub fn get_item<'a>(&'a self, filename : &String)->Option<&'a DirItem> {
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
pub struct DirItem{
    pub filename : String,
    pub name : String,
    pub start_cluster : usize,
    pub block_idx : usize,
    pub size : usize,
    pub idx : usize,
    pub len : usize,
    attr : u8,
}

impl DirItem {
    pub fn new(name : String, st_cluster : usize, attr : u8,
            block_idx : usize, size : usize, idx : usize, len : usize) -> Self {
        let mut s = String::new();
        for c in name.bytes(){
            if c as char != ' '{
                s.push(c as char);
            }
        }
        let filename = s.clone();
        DirItem{
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

/// ## 获取目录的文件树
/// block_idx : 磁盘号
/// cluster : 传入目录的簇号
pub fn get_directory(block_idx : usize, cluster : usize) ->Option<Directory> {
    if let Some(mgr) = get_mgr(block_idx){
        match mgr {
            Mgr::FAT(fat) => {
                fat.get_directory(cluster)
            }
        }
    }
    else{
        None
    }
}

use alloc::{prelude::v1::*};
use crate::{filesystem::{Mgr, interface::get_mgr, require::{DoDirectory}}, libs::str::{convert_to_usize, make_shortname}, uart};
use super::{file::{File}, super::{fat32::Attribute}};


