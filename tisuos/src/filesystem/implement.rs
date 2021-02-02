//! # 文件系统 trait 实现
//! 
//! 2021年1月30日 zg


/// 
/// ## FAT32
/// 

impl DoIO for FATManger {
    fn read(&mut self, block_idx : usize, cluster : usize, offset : usize, len : usize)->Option<Block<u8>> {
        let rt = Block::new(len);
        let mut cluster = cluster;
        let mut size = len;
        let mut item = self.get_fat_item(cluster).unwrap();
        let mut idx = 0;
        let mut offset = self.cluster_start_addr + cluster * self.cluster_size + offset;
        while item.has_next(){
            let tmp_size = min(size, self.cluster_size);
            sync_read_buffer(block_idx, &rt, idx, tmp_size as u32, offset);
            idx += tmp_size;
            cluster = item.get_next_item_num() as usize;
            size -= tmp_size;
            item = self.get_fat_item(cluster).unwrap();
            offset = self.cluster_start_addr + cluster * self.cluster_size;
        }
        sync_read_buffer(block_idx, &rt, idx,
            min(size, self.cluster_size) as u32, offset);
        Some(rt)
    }

    fn write(&mut self, block_idx : usize, cluster : usize, offset : usize, len : usize, content : &Block<u8>) {
        let mut cluster = cluster;
        let mut size = len;
        let mut item = self.get_fat_item(cluster).unwrap();
        let mut idx = 0;
        let mut offset = self.cluster_start_addr + cluster * self.cluster_size + offset;
        while item.has_next(){
            let tmp_size = min(size, self.cluster_size);
            sync_write_buffer(block_idx, content, idx, tmp_size as u32, offset);
            idx += tmp_size;
            cluster = item.get_next_item_num() as usize;
            size -= tmp_size;
            item = self.get_fat_item(cluster).unwrap();
            offset = self.cluster_start_addr + cluster * self.cluster_size;
        }
        sync_write_buffer(block_idx, content, idx,
            min(size, self.cluster_size) as u32, offset);
    }
}

impl DoDirectory for FATManger{
    fn get_directory(&mut self, cluster : usize)->Option<directory::Directory> {
        if let Some(items) = self.get_dir_items(cluster){
            let mut files = Vec::<DirItem>::new();
            let mut filename = String::new();
            let mut len = 0;
            let mut idx = 0;
            for item in items{
                if item.attr == Attribute::LongName.val(){
                    if len == 0{
                        idx = item.idx;
                    }
                    filename = item.filename + &filename[..];
                    len += 1;
                }
                else if item.attr != Attribute::LongName.val() {
                    len += 1;
                    if filename.len() <= 0{
                        filename = item.filename;
                        len = 1;
                        idx = item.idx;
                    }
                    files.push(
                        DirItem::new(
                            filename.clone(),
                            item.start_cluster,
                            item.attr,
                            self.block_idx,
                            item.size,
                            idx,
                            len
                        )
                    );
                    len = 0;
                    filename.clear();
                }
                else{
                    filename.clear();
                }
            }
            return Some(
                directory::Directory::new(
                    String::from("/"), files,cluster, self.block_idx
                )
            );
        }
        else {
            None
        }
    }
    fn get_root_directory(&mut self)->Option<directory::Directory> {
        self.get_directory(2)
    }

    fn create_directory(&mut self, tree : &directory::Directory, name : &String)->bool {
        let cluster = self.find_free_fat_item(1).unwrap();
        let shortname = self.make_shortname(&name).to_uppercase();
        if tree.has_shortname(&shortname){
            return false;
        }
        let check = check_sum(&shortname);
        let names = self.split_long_name(name);
        let mut cnt = names.len();
        let mut idx = self.find_free_dir_item_index(tree.start_cluster, cnt + 1).unwrap();
        self.set_fat_item(cluster, FLAG_END);
        for name in names.iter(){
            let item = FATLongDirItem::new(
                name.clone(), cnt, check, cnt == names.len());
            self.set_long_dir_item(tree.start_cluster, idx, item);
            idx += 1;
            cnt -= 1;
        }
        self.clear_directory(cluster);
        let item = FATShortDirItem::new(shortname.clone(), 
            Attribute::SubDir.val(), cluster, 0);
        self.set_short_dir_item(tree.start_cluster, idx, item);
        let item = FATShortDirItem::new(".       ".to_string(),
                Attribute::SubDir.val(), cluster, 0);
        self.set_short_dir_item(cluster, 0, item);
        let item = FATShortDirItem::new("..      ".to_string(), 
                    Attribute::SubDir.val(), tree.start_cluster, 0);
        self.set_short_dir_item(cluster, 1, item);
        true
    }

    fn clear_directory(&mut self, cluster : usize) {
        let st = self.cluster_start_addr + cluster * self.cluster_size;
        sync_write_zero(self.block_idx, self.cluster_size as u32, st);
    }

    fn delete_file(&mut self, tree : &directory::Directory, name : &String)->bool {
        let shortname = make_shortname(&name);
        if !tree.has_shortname(&shortname){
            false
        }
        else{
            let item = tree.get_item(name).unwrap();
            if item.is_dir(){
                return false;
            }
            println!("get item {}", item.filename);
            self.delete_sequence_fat_item(item.start_cluster);
            self.delete_sequence_dir_item(tree.start_cluster, item.idx, item.len);
            true
        }
    }
}

impl DoInfo for FATManger{
    fn get_total_size(&self) ->usize{
        self.bpb.total_sector2 as usize * self.bpb.bytes_per_sector as usize
    }
    fn get_used_size(&self) ->usize {
        let mut rt : usize = 0;
        let size = self.bpb.fat_sector_num / self.bpb.fat_num as u32
            * self.bpb.bytes_per_sector as u32;
        let cnt = size as usize / size_of::<FATItem>() as usize;
        for i in 0..cnt{
            if let Some(item) = self.get_fat_item(i as usize){
                if !item.is_free(){
                    rt += 1;
                }
            }
        }
        rt * self.bpb.bytes_per_sector as usize * self.bpb.sector_per_cluster as usize
    }
    
}



use core::{cmp::min, mem::size_of};

use crate::{libs::str::{check_sum, make_shortname}, memory::block::{Block}, uart, virtio::buffer::{sync_read_buffer, sync_write_buffer, sync_write_zero}};

use super::{fat32::{Attribute, FATItem, FATLongDirItem, FATManger, FATShortDirItem, FLAG_END}, filetree::directory,
    filetree::directory::DirItem, require::{DoDirectory, DoIO, DoInfo}};
use alloc::{prelude::{v1::*}};

