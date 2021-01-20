#![allow(dead_code)]
//! # FAT32
//! 用于分析 FAT32 文件系统
//! 2020年12月18日 zg

use alloc::{prelude::v1::*};

pub const FLAG_END : u32 = 0x0fffffff;

//
// 工具结构体 
// 

/// ## FATInfo
/// 处理 FAT32 格式的磁盘信息
#[derive(Clone, Copy)]
#[repr(packed)]
pub struct FATInfo{
    jump1   : u8,
    j2      : u8,
    j3      : u8,
    oem     : [u8;8],
    bpb     : BPB,
    ext     : Extend,
}
impl FATInfo {
    /// 获取 FAT 表起始地址
    pub fn get_fat_addr(&self) -> usize{
        self.bpb.reserved_sector as usize * self.bpb.bytes_per_sector as usize
    }
    pub fn get_fat_size(&self) -> usize{
        self.bpb.fat_num as usize * self.bpb.fat_sector_num as usize * self.bpb.bytes_per_sector as usize
    }
    pub fn get_directory_addr(&self) -> usize{
        self.get_fat_addr() + self.get_fat_size()
    }
    pub fn get_fat_item(&self, num : usize) -> Option<FATItem>{
        if num >= self.get_fat_size() as usize / size_of::<FATItem>(){
            None
        }
        else{
            let base = self.get_fat_addr() as *mut FATItem;
            unsafe {
                let t = base.add(num);
                Some((*t).clone())
            }
        }
    }
    pub fn is_fat(&self)->bool{
        let mut s = String::new();
        for c in self.oem.iter(){
            s.push(*c as char);
        }
        s.contains("fat")
    }
}
/// ## FATManger
/// 初始化获得一个磁盘的 FAT32 信息，提供目录项及文件的读取
/// 使用缓存保存常用的 fat 表内容
pub struct FATManger{
    /// FAT 表起始地址
    fat_addr : usize,
    /// 两个 FAT 表总大小，字节为单位
    fat_total_size : usize,
    fat_num : usize,
    total_cluster : usize,
    /// 簇占字节大小
    pub cluster_size : usize,
    root_dir_cluster_addr : usize,
    /// 第一个簇开始的地址
    pub cluster_start_addr : usize,
    pub block_idx : usize,
    pub bpb : BPB,
}
static mut DEBUG : bool = false;
impl FATManger {
    pub fn new(info : &FATInfo, block_idx : usize) -> Self{
        let byte_per_sector = info.bpb.bytes_per_sector as usize;
        let sector_per_cluster = info.bpb.sector_per_cluster as usize;
        let total_cluster = info.bpb.total_sector2 as usize / sector_per_cluster;
        let used_cluster = info.get_directory_addr() / byte_per_sector / sector_per_cluster;
        let total_cluster = total_cluster - used_cluster + 1;
        let cluster_size = info.bpb.sector_per_cluster as usize * byte_per_sector;
        FATManger{
            fat_addr : info.get_fat_addr(),
            fat_total_size : info.get_fat_size(),
            total_cluster : total_cluster,
            fat_num : info.bpb.fat_num as usize,
            root_dir_cluster_addr : info.get_directory_addr(),
            cluster_start_addr : info.get_directory_addr() - cluster_size * 2,
            cluster_size : cluster_size,
            block_idx : block_idx,
            bpb : info.bpb.clone()
        }
    }
    // 获取 fat 表项
    pub fn get_fat_item(&mut self, cluster : usize) ->Option<FATItem> {
        let addr = self.fat_addr as usize + cluster * size_of::<FATItem>();
        let b = new_block(size_of::<FATItem>());
        // if unsafe {DEBUG} {
        //     println!("before read fat");
        // }
        sync_read_buffer(self.block_idx, 
            b.addr, size_of::<FATItem>() as u32, addr);
        // if unsafe {DEBUG} {
        //     println!("after read fat");
        // }
        unsafe{
            Some(
                (*(b.addr as *mut FATItem)).clone()
            )
        }
    }
    /// 获取目录项
    pub fn get_dir_items(&mut self, sub_dir : usize) -> Option<Vec::<FatDirItem>>{
        let t = sub_dir;
        if t >= 2 {
            let items = self.get_all_cluster(t as usize);
            if let Some(list) = items{
                self.get_dir_item(&list)
            }
            else{
                None
            }
        }
        else{
            None
        }
    }
    pub fn split_long_name(&self, name : &String)->Vec<String>{
        let mut names = Vec::<String>::new();
        let mut name = name.clone();
        while name.len() > 13{
            names.push(name.split_at(13).0.to_string());
            name = name.split_at(13).1.to_string();
        }
        names.push(name);
        names.reverse();
        names
    }
    pub fn make_shortname(&self, name : &String)->String{
        let tmp = name.clone().to_uppercase();
        let mut name = String::new();
        for c in tmp.split(' '){
            name += c;
        }
        let (head, tail) = split_back(&name, '.');
        let mut s = String::new();
        if head.len() < 8{
            s = head.to_string();
            while s.len() < 8{
                s += " ";
            }
        }
        else {
            s = head.split_at(6).0.to_string();
            s += "~1";
        }
        s += tail;
        while s.len() < 11{
            s += " ";
        }
        s
    }
    pub fn expend_fat_item(&mut self, cluster : usize){
        let item = self.get_fat_item(cluster).unwrap();
        if item.is_end(){
            let next = self.find_free_fat_item(1).unwrap();
            self.set_fat_item(cluster, next as u32);
            self.set_fat_item(next, FLAG_END);
        }
    }
    pub fn set_fat_item(&mut self, cluster : usize, val : u32){
        let b = new_block(4);
        unsafe {
            (b.addr as *mut u32).write_volatile(val);
        }
        let offset = cluster * size_of::<FATItem>() + self.fat_addr;
        println!("write offset {:x}", offset);
        sync_write_buffer(self.block_idx, b.addr, 4, offset);
    }
    pub fn find_free_fat_item(&mut self, num : usize)->Option<usize>{
        let mut cluster : usize = 2;
        let mut cnt = 0;
        let mut idx = 0;
        let mut item = self.get_fat_item(cluster).unwrap();
        while (!item.is_free() || cnt < num) && cluster < self.total_cluster{
            if item.is_free(){
                cnt += 1;
                if cnt == 1{
                    idx = cluster;
                }
            }
            else{
                cnt = 0;
            }
            cluster += 1;
            item = self.get_fat_item(cluster).unwrap();
        }
        if item.is_free(){
            Some(idx)
        }
        else{
            None
        }
    }
    pub fn set_long_dir_item(&mut self, cluster : usize, idx : usize, item : FATLongDirItem){
        let st = self.root_dir_cluster_addr;
        let clusters = self.get_all_cluster(cluster).unwrap();
        let i = idx / (self.cluster_size / size_of::<FATLongDirItem>());
        if i >= clusters.len(){
            return;
        }
        let cluster = clusters[i];
        let addr = st + (cluster - 2) * self.cluster_size + idx * size_of::<FATLongDirItem>();
        let buffer = new_block(size_of::<FATLongDirItem>());
        unsafe {
            (buffer.addr as *mut FATLongDirItem).write_volatile(item);
        }
        sync_write_buffer(self.block_idx, buffer.addr,
            size_of::<FATShortDirItem>() as u32, addr);
    }
    pub fn set_short_dir_item(&mut self, cluster : usize, idx : usize, item : FATShortDirItem){
        let st = self.root_dir_cluster_addr;
        let clusters = self.get_all_cluster(cluster).unwrap();
        let i = idx / (self.cluster_size / size_of::<FATShortDirItem>());
        if i >= clusters.len(){
            return;
        }
        let cluster = clusters[i];
        let addr = st + (cluster - 2) * self.cluster_size + idx * size_of::<FATShortDirItem>();
        let buffer = new_block(size_of::<FATShortDirItem>());
        unsafe {
            (buffer.addr as *mut FATShortDirItem).write_volatile(item);
        }
        sync_write_buffer(self.block_idx, buffer.addr,
            size_of::<FATShortDirItem>() as u32, addr);
    }
    pub fn get_dir_item_by_index(&mut self, cluster : usize, idx : usize)->Option<FATShortDirItem>{
        let st = self.root_dir_cluster_addr;
        let clusters = self.get_all_cluster(cluster).unwrap();
        let i = idx / self.cluster_size / size_of::<FATShortDirItem>();
        if i >= clusters.len(){
            return None;
        }
        let cluster = clusters[i];
        let addr = st + (cluster - 2) * self.cluster_size;
        let buffer = new_block(self.cluster_size);
        sync_read_buffer(self.block_idx, buffer.addr, self.cluster_size as u32, addr);
        unsafe {
            let item = (buffer.addr as *mut FATShortDirItem).add(idx);
            return Some((*item).clone());
        }
    }
    /// ## 删除 fat 表中的簇及其后续
    pub fn delete_sequence_fat_item(&mut self, cluster : usize){
        let clusters = self.get_all_cluster(cluster).unwrap();
        for cluster in clusters{
            self.set_fat_item(cluster, 0);
        }
    }
    /// ## 删除目录中的项
    pub fn delete_sequence_dir_item(&mut self, cluster : usize, idx : usize, len : usize){
        let st = self.cluster_start_addr;
        let clusters = self.get_all_cluster(cluster).unwrap();
        let buffer = new_block(size_of::<FATShortDirItem>());
        for i in 0..len{
            let ii = (idx + i) / (self.cluster_size / size_of::<FATShortDirItem>());
            if ii >= clusters.len(){
                return;
            }
            let cluster = clusters[ii];
            let addr = st + cluster * self.cluster_size + (idx + i) * size_of::<FATShortDirItem>();
            sync_write_buffer(self.block_idx, buffer.addr,
                size_of::<FATShortDirItem>() as u32, addr);
        }
        
    }
    /// ## 获取目录中的空项下标
    /// 传入目录的起始簇号，返回空项在整个目录中的下标
    pub fn find_free_dir_item_index(&mut self, cluster : usize, num : usize)->Option<usize>{
        let st = self.root_dir_cluster_addr;
        let step = size_of::<FATShortDirItem>();
        let clusters = self.get_all_cluster(cluster).unwrap();
        let mut base = 0;
        let mut cnt = 0;
        let mut record = 0;
        for cluster in clusters {
            let addr = st + (cluster - 2) * self.cluster_size;
            let buffer = new_block(self.cluster_size);
            sync_read_buffer(self.block_idx, buffer.addr, self.cluster_size as u32, addr);
            for idx in (0..self.cluster_size).step_by(step){
                unsafe {
                    let item = buffer.addr.add(idx) as *mut FATShortDirItem;
                    if (*item).is_empty() ||(*item).is_delete() {
                        if cnt == 0{
                            record = idx;
                        }
                        cnt += 1;
                        if cnt >= num {
                            return Some(base / step + record / step);
                        }
                    }
                    else{
                        cnt = 0;
                    }
                }
            }
            base += self.cluster_size;
        }
        None
    }
    /// 根据目录簇号获取里面的目录项
    fn get_dir_item(&mut self, clusters : &Vec::<usize>) ->Option<Vec::<FatDirItem>> {
        let mut rt = Vec::<FatDirItem>::new();
        let st = self.root_dir_cluster_addr;
        let step = size_of::<FATShortDirItem>();
        for cluster in clusters {
            let addr = st + (cluster - 2) * self.cluster_size;
            let buffer = new_block(self.cluster_size);
            sync_read_buffer(self.block_idx, buffer.addr, self.cluster_size as u32, addr);
            for idx in (0..self.cluster_size).step_by(step){
                unsafe {
                    let item = buffer.addr.add(idx) as *mut FATShortDirItem;
                    if !(*item).is_empty() && !(*item).is_delete() {
                        if (*item).is_long_dir(){
                            rt.push((*(item as *mut FATLongDirItem)).to_dir_item(idx / step));
                        }
                        else{
                            rt.push((*item).to_dir_item(idx / step));
                        }
                    }
                }
            }
        }
        Some(rt)
    }
    /// 根据起始簇号获取整个簇链
    fn get_all_cluster(&mut self, start_cluster : usize) ->Option<Vec::<usize>> {
        let mut rt = Vec::<usize>::new();
        let mut num = start_cluster;
        unsafe {
            DEBUG = true;
        }
        // println!("start {} {:x}", num, self.fat_addr);
        loop {
            if let Some(item) = self.get_fat_item(num){
                // println!("next {:x}", item.item);
                if !item.is_end(){
                    if item.is_bad(){
                        return None;
                    }
                    else{
                        num = item.get_next_item_num() as usize;
                    }
                }
                else {
                    rt.push(num);
                    break;
                }
            }
            else{
                break;
            }
        }
        Some(rt)
    }

}
#[derive(Clone)]
pub struct FATItem{
    item : u32,
}
impl FATItem {
    pub fn is_free(&self) ->bool {
        self.item == 0
    }
    pub fn is_reserved(&self) ->bool {
        self.item == 1
    }
    pub fn is_end(&self) ->bool {
        self.item >= 0x0ffffff8 && self.item <= 0x0fffffff
    }
    pub fn is_bad(&self) ->bool {
        self.item == 0x0ffffff7
    }
    pub fn get_next_item_num(&self) -> u32{
        self.item
    }
    pub fn has_next(&self)->bool{
        self.item >= 2 && self.item < 0x0ffffff7
    }
    pub fn is_item(&self) ->bool {
        self.item >= 2 && self.item <= 0x0fffffff
    }
}

pub struct FatDirItem{
    pub filename : String,
    pub attr : u8,
    pub start_cluster : usize,
    pub size : usize,
    pub idx : usize,
}
impl FatDirItem {
    pub fn new(filename : String, attr : u8, start_cluster : usize, size : usize, idx : usize)->Self{
        Self{
            filename : filename,
            attr : attr,
            start_cluster : start_cluster,
            size : size,
            idx : idx,
        }
    }
}
#[repr(packed)]
pub struct FATShortDirItem{
    pub filename : u64,
    ext_name : [u8;3],
    pub attr : u8,
    reserved : u8,
    create_time_ext : u8, // 10 毫秒位
    create_time : u16,
    create_date : u16, // 16
    last_access_date : u16,
    start_cluster_high : u16,
    last_change_time : u16,
    last_change_date : u16,
    start_cluster_low : u16,
    pub file_length : u32,
}

impl Clone for FATShortDirItem {
    fn clone(&self) -> Self {
        FATShortDirItem{
            filename : self.filename,
            ext_name : self.ext_name,
            attr : self.attr.clone(),
            reserved : self.reserved,
            create_time_ext : self.create_time_ext, // 10 毫秒位
            create_time : self.create_time,
            create_date : self.create_date,
            last_access_date : self.last_access_date,
            start_cluster_high : self.start_cluster_high,
            last_change_time : self.last_change_time,
            last_change_date : self.last_change_date,
            start_cluster_low : self.start_cluster_low,
            file_length : self.file_length,
        }
    }
}
impl FATShortDirItem {
    pub fn new(name : String, attr : u8, cluster : usize, len : usize)->Self{
        let filename = name.to_uppercase();
        let t : Vec<&str> = filename.split('.').collect();
        let mut filename = t[0].to_string();
        while filename.len() < 8{
            filename.push(' ');
        }
        let mut ext = [' ', ' ', ' '];
        if t.len() > 1{
            let mut s = t.last().unwrap().to_string();
            s.push(' ');
            s.push(' ');
            for (i, c) in s.bytes().enumerate(){
                if i >= 3{
                    break;
                }
                ext[i] = c as char;
            }
        }
        if attr == Attribute::SubDir.val(){
            filename = name;
        }
        Self{
            filename : to_u64(filename),
            ext_name : [ext[0] as u8, ext[1] as u8, ext[2] as u8],
            attr : attr,
            reserved : 0,
            create_time_ext : 0x2a,
            create_time : 0x3410,
            create_date : 0x5195,
            last_access_date : 0x5195,
            start_cluster_high : (cluster >> 16) as u16 & 0xffff,
            last_change_date : 0x5159,
            last_change_time : 0x3410,
            start_cluster_low : (cluster & 0xffff) as u16,
            file_length : len as u32,
        }
    }
    pub fn is_file(&self) ->bool {
        self.attr == Attribute::Archive.val()
    }
    pub fn is_dir(&self) ->bool {
        self.attr == Attribute::SubDir.val()
    }
    pub fn is_long_dir(&self)->bool{
        self.attr == Attribute::LongName.val()
    }
    pub fn get_cluster(&self) ->u32{
        (self.start_cluster_high as u32) << 16 | self.start_cluster_low as u32
    }
    pub fn is_empty(&self) ->bool {
        self.attr == Attribute::Empty.val()
    }
    pub fn is_delete(&self)->bool{
        self.filename & 0xff == 0xe5
    }
    pub fn get_filename(&self) ->String {
        from_u64(self.filename)
    }
    pub fn get_ext(&self) ->String {
        let mut ext = String::new();
        for i in self.ext_name.iter(){
            ext.push(*i as char);
        }
        ext
    }
    pub fn to_dir_item(&self, idx : usize)->FatDirItem{
        FatDirItem::new(self.get_filename() + &self.get_ext()[..], 
            self.attr, self.get_cluster() as usize, self.file_length as usize, idx)
    }
}
#[repr(packed)]
pub struct FATLongDirItem{
    pub flag : u8,
    pub name1 : [u16;5],
    pub attr : u8,
    reserved : u8,
    pub check : u8,
    pub name2 : [u16;6],
    pub start_cluster : u16, // 常置 0
    pub name3 : [u16;2]
}
impl FATLongDirItem{
    pub fn new(filename : String, num : usize, check : u8, last : bool)->Self{
        let mut name1 : [u16;5] = [0xffff;5];
        let mut name2 : [u16;6] = [0xffff;6];
        let mut name3 : [u16;2] = [0xffff;2];
        let mut idx = 0;
        for (i, c) in filename.bytes().enumerate(){
            if i < 5{
                name1[i] = c as u16;
            }
            else if i < 11 {
                name2[i - 5] = c as u16;
            }
            else if i < 13{
                name3[i - 11] = c as u16;
            }
            idx = i;
        }
        idx += 1;
        if idx < 5{
            name1[idx] = 0;
        }
        else if idx < 11 {
            name2[idx - 5] = 0;
        }
        else if idx < 13{
            name3[idx - 11] = 0;
        }
        let mut flag = num as u8;
        if last {
            flag |= 0x1 << 6;
        }
        Self{
            flag : flag,
            name1 : name1,
            attr : 0x0f,
            reserved : 0,
            check : check,
            name2 : name2,
            start_cluster : 0,
            name3 : name3,
        }
    }
    pub fn to_dir_item(&self, idx : usize)->FatDirItem{
        FatDirItem::new(self.get_name(), self.attr, 0, 0, idx)
    }
    pub fn get_name(&self)->String{
        let mut name = String::new();
        let mut t = Vec::<u16>::new();
        unsafe {
            for i in self.name1.iter(){
                if *i == 0 || *i == 0xffff{
                    break;
                }
                t.push(*i);
            }
            for i in self.name2.iter(){
                if *i == 0 || *i == 0xffff{
                    break;
                }
                t.push(*i);
            }
            for i in self.name3.iter(){
                if *i == 0 || *i == 0xffff{
                    break;
                }
                t.push(*i);
            }
        }
        let res = String::from_utf16(&t[..]).unwrap();
        name.push_str(&res[..]);
        name
    }
}
#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum Attribute{
    Empty = 0,
    ReadOnly = 1,
    Hidden = 2,
    System = 4,
    VolumeLabel = 8,
    SubDir = 16,
    Archive = 32,
    LongName = 1 | 2 | 4 | 8,
}
impl Attribute{
    pub fn val(&self) ->u8 {
        self.clone() as u8
    }
}
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct BPB {
    pub bytes_per_sector    : u16,
    pub sector_per_cluster  : u8,
    pub reserved_sector     : u16, // 第一个簇之前的所有扇区都是保留扇区
    pub fat_num             : u8,
    pub root_entry          : u16, // fat32 不使用
    pub total_sector1       : u16, // 如果为 0，使用 total_sector2中的值
    pub desc                : u8, // 提供有关媒体被使用的信息。值0xF8表示硬盘，0xF0表示高密度的3.5寸软盘
    pub sector_per_fat_16   : u16, // 16 位下
    pub sector_per_track    : u16, // 每磁道扇区
    pub track_num           : u16,  // 磁头数
    pub hidden_sector       : u32, // 引导扇区之前的扇区数，无分区为 0
    pub total_sector2       : u32,
    pub fat_sector_num      : u32,
}
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Extend{
    ext_flag : u16,
    filesystem_version  : u16,
    root_cluster_start  : u32,
    filesystem_info     : u16,
    backup_boot_sector  : u16,
    reserved1           : u64,
    reserved2           : u32
}

use core::{mem::size_of};
use crate::{libs::str::{from_u64, split_back, to_u64}, memory, uart, virtio::buffer::{sync_read_buffer, sync_write_buffer}};
use memory::block::new_block;


