//! 缓冲区
//! 因为磁盘读写太慢，采用缓存的方式，内核所有磁盘操作先经过这层封装
//! 2020年12月31日 zg

use core::cmp::min;

use crate::{memory::{block::Block}, sync::{ReadWriteMutex}};

const BUFFER_SIZE : usize = 1024 * 4096; // 4 MB
const CACHE_SIZE : usize = 4;
static mut LOCK : ReadWriteMutex = ReadWriteMutex::new();
static mut COUNTER : usize = 1;
static mut CACHE : Option<Vec<Buffer>> = None;

pub struct Buffer{
    mutex : ReadWriteMutex,
    cnt : usize,
    block_idx : usize,
    idx : usize,
    size : usize,
    addr : Block<u8>,
    is_write : bool,
}

impl Buffer {
    pub fn new(idx : usize)->Self{
        let b = Block::new(BUFFER_SIZE);
        Self{
            mutex : ReadWriteMutex::new(),
            cnt : 0,
            block_idx : 0,
            idx : idx,
            size : BUFFER_SIZE,
            addr : b,
            is_write : false,
        }
    }
    /// ### 拷贝至指定 Block
    pub fn copy_to(&mut self, idx : usize, target : &Block<u8>, offset : usize, len : usize){
        unsafe {
            COUNTER += 1;
        }
        self.cnt = unsafe{ COUNTER };
        let size = min(self.idx + self.size - idx, len);
        let st = idx - self.idx;
        self.mutex.read();
        self.addr.copy_to(st,target, offset, size);
        self.mutex.unlock();
    }
    /// ### 缓存是否包含给出的磁盘地址
    pub fn is_contain(&self, idx : usize)->bool{
        self.idx <= idx && self.idx + self.size > idx
    }
    pub fn copy_len(&self, idx : usize)->usize{
        self.idx + self.size - idx
    }
    /// ### 从磁盘读取更新缓存内容
    pub fn refresh(&mut self, block_idx : usize, idx : usize){
        let idx = idx / BUFFER_SIZE * BUFFER_SIZE;
        println!("refresh idx {:x}", idx);
        // 暂时没有写入要求，如果需要写入，则取消注释
        // self.write_down();
        self.block_idx = block_idx;
        self.idx = idx;
        self.mutex.write();
        sync_read(block_idx, self.addr.get_addr() as *mut u8, self.size as u32, idx);
        self.mutex.unlock();
    }
    /// ### 从指定 Block 拷贝
    pub fn copy_from(&mut self, idx : usize, src : &Block<u8>, offset : usize, len : usize){
        unsafe {
            COUNTER += 1;
        }
        self.cnt = unsafe{ COUNTER };
        let size = min(self.idx + self.size - idx, len);
        let st = idx - self.idx;
        self.mutex.write();
        self.addr.copy_from(st, src, offset, size);
        self.is_write = true;
        self.mutex.unlock();
    }
    /// ### 将缓存内容写入磁盘
    pub fn write_down(&mut self){
        if !self.is_write{
            return;
        }
        println!("buffer.rs writedown {:x}", self.idx);
        self.mutex.write();
        self.is_write = false;
        self.mutex.unlock();
        self.mutex.read();
        sync_write(self.block_idx, self.addr.get_addr() as *mut u8, self.size as u32, self.idx);
        // println!("buffer.rs wirtedown finish");
        self.mutex.unlock();
    }
    /// ### 清零
    pub fn zero(&mut self, idx : usize, len : usize){
        unsafe {
            COUNTER += 1;
        }
        self.cnt = unsafe{ COUNTER };
        let size = min(self.idx + self.size - idx, len);
        let st = idx - self.idx;
        self.mutex.write();
        self.addr.set(st, 0, size);
        // self.addr.get_addr().add(st).write_bytes(0, size);
        self.is_write = true;
        self.mutex.unlock();
    }
}

pub fn init(){
    unsafe {
        let mut v = Vec::<Buffer>::new();
        for i in 0..CACHE_SIZE {
            let idx = i * BUFFER_SIZE;
            let mut buf = Buffer::new(idx);
            buf.refresh(0, idx);
            v.push(buf);
        }
        CACHE = Some(v);
    }
}

/// ## 定时写入磁盘
/// 应该挂载在线程中
#[allow(dead_code)]
pub fn write_down_handler(){
    loop {
        unsafe {
            asm!("wfi"::::"volatile");
            LOCK.read();
            if let Some(cache) = &mut CACHE{
                for buf in cache{
                    buf.write_down();
                }
            }
            LOCK.unlock();
        }
    }
}

pub fn sync_read_buffer(block_idx : usize, buffer : &Block<u8>, st : usize, size : u32, offset : usize){
    let mut idx = offset;
    let mut offset = st;
    let mut len = size as usize;
    unsafe {
        LOCK.write();
    }
    while len > 0{
        if let Some(buf) = find_buffer(idx){
            buf.copy_to(idx, buffer, offset, len);
            let count = min(len, buf.copy_len(idx));
            len -= count;
            offset += count;
            idx += count;
        }
        else{
            new_buffer(block_idx, idx);
        }
    }
    unsafe {
        LOCK.unlock();
    }
}

pub fn sync_write_buffer(block_idx : usize, buffer : &Block<u8>, st : usize, size : u32, offset : usize){
    let mut idx = offset;
    let mut offset = st;
    let mut len = size as usize;
    unsafe {
        LOCK.write();
    }
    while len > 0{
        if let Some(buf) = find_buffer(idx){
            buf.copy_from(idx, buffer, offset, len);
            let count = min(len, buf.copy_len(idx));
            len -= count;
            offset += count;
            idx += count;
        }
        else{
            new_buffer(block_idx, idx);
        }
    }
    unsafe {
        LOCK.unlock();
    }
}

pub fn sync_write_zero(block_idx : usize, size : u32, offset : usize){
    let mut idx = offset;
    let mut len = size as usize;
    unsafe {
        LOCK.write();
    }
    while len > 0{
        if let Some(buf) = find_buffer(idx){
            buf.zero(idx, len);
            let count = min(len, buf.copy_len(idx));
            len -= count;
            idx += count;
        }
        else{
            new_buffer(block_idx, idx);
        }
    }
    unsafe {
        LOCK.unlock();
    }
}

fn new_buffer(block_idx : usize, idx : usize){
    unsafe {
        let mut mix = 0;
        if let Some(cache) = &mut CACHE{
            for (idx, buf) in cache.iter().enumerate() {
                if buf.cnt < cache[mix].cnt{
                    mix = idx;
                }
            }
            let n = cache[mix].cnt;
            COUNTER -= n;
            if let Some(buf) = cache.get_mut(mix){
                buf.refresh(block_idx, idx);
            }
            for buf in cache.iter_mut(){
                buf.cnt -= n;
            }
        }
    }
}

fn find_buffer<'a>(idx : usize)->Option<&'a mut Buffer>{
    unsafe {
        if let Some(cache) = &mut CACHE{
            for buf in cache{
                if buf.is_contain(idx){
                    return Some(buf);
                }
            }
        }
        None
    }
}


use alloc::{prelude::v1::*};
use crate::uart;
use super::device::{sync_read, sync_write};

