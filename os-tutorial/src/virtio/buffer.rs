//! 缓冲区
//! 因为磁盘读写太慢，采用缓存的方式，内核所有磁盘操作先经过这层封装
//! 2020年12月31日 zg

use core::cmp::min;

use crate::{memory::{page::{PAGE_SIZE, alloc_kernel_page}}, sync::{ReadWriteMutex}};

const BUFFER_SIZE : usize = 1024 * 4096; // 4 MB
const CACHE_SIZE : usize = 2;
static mut LOCK : ReadWriteMutex = ReadWriteMutex::new();
static mut COUNTER : usize = 1;
static mut CACHE : Option<Vec<Buffer>> = None;
pub static mut DEBUG : bool = false;
pub struct Buffer{
    mutex : ReadWriteMutex,
    cnt : usize,
    block_idx : usize,
    idx : usize,
    size : usize,
    addr : *mut u8,
    is_write : bool,
}

impl Buffer {
    pub fn new(idx : usize)->Self{
        let n = (BUFFER_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;
        Self{
            mutex : ReadWriteMutex::new(),
            cnt : 0,
            block_idx : 0,
            idx : idx,
            size : BUFFER_SIZE,
            addr : alloc_kernel_page(n),
            is_write : false,
        }
    }
    pub fn copy_to(&mut self, target : *mut u8, idx : usize, len : usize){
        unsafe {
            COUNTER += 1;
        }
        self.cnt = unsafe{ COUNTER };
        let size = min(self.idx + self.size - idx, len);
        let st = idx - self.idx;
        self.mutex.read();
        unsafe {
            // if DEBUG {
            //     println!("start copy addr {:x} size {}", target as usize, size);
            // }
            target.copy_from(self.addr.add(st), size);
            // if DEBUG {
            //     println!("end copy");
            // }
        }
        self.mutex.unlock();
    }
    pub fn is_contain(&self, idx : usize)->bool{
        self.idx <= idx && self.idx + self.size > idx
    }
    pub fn copy_len(&self, idx : usize)->usize{
        self.idx + self.size - idx
    }
    pub fn refresh(&mut self, block_idx : usize, idx : usize){
        let idx = idx / BUFFER_SIZE * BUFFER_SIZE;
        println!("refresh idx {:x}", idx);
        self.block_idx = block_idx;
        self.idx = idx;
        self.mutex.write();
        sync_read(block_idx, self.addr, self.size as u32, idx);
        self.mutex.unlock();
    }
    pub fn copy_from(&mut self, src : *mut u8, idx : usize, len : usize){
        unsafe {
            COUNTER += 1;
        }
        self.cnt = unsafe{ COUNTER };
        let size = min(self.idx + self.size - idx, len);
        let st = idx - self.idx;
        self.mutex.write();
        unsafe {
            self.addr.add(st).copy_from(src, size);
        }
        self.is_write = true;
        self.mutex.unlock();
    }
    pub fn write_down(&mut self){
        if !self.is_write{
            return;
        }
        println!("buffer.rs writedown {:x}", self.idx);
        self.mutex.write();
        self.is_write = false;
        self.mutex.unlock();
        self.mutex.read();
        sync_write(self.block_idx, self.addr, self.size as u32, self.idx);
        // println!("buffer.rs wirtedown finish");
        self.mutex.unlock();
    }
    pub fn zero(&mut self, idx : usize, len : usize){
        unsafe {
            COUNTER += 1;
        }
        self.cnt = unsafe{ COUNTER };
        let size = min(self.idx + self.size - idx, len);
        let st = idx - self.idx;
        self.mutex.write();
        unsafe {
            self.addr.add(st).write_bytes(0, size);
        }
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

pub fn sync_read_buffer(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
    let mut ptr = buffer;
    let mut idx = offset;
    let mut len = size as usize;
    unsafe {
        LOCK.write();
    }
    while len > 0{
        if let Some(buf) = find_buffer(idx){
            buf.copy_to(ptr, idx, len);
            let l = min(len, buf.copy_len(idx));
            len -= l;
            unsafe {
                ptr = ptr.add(l);
            }
            idx += l;
        }
        else{
            new_buffer(block_idx, idx);
        }
    }
    unsafe {
        LOCK.unlock();
    }
}

pub fn sync_write_buffer(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
    let mut ptr = buffer;
    let mut idx = offset;
    let mut len = size as usize;
    unsafe {
        LOCK.write();
    }
    while len > 0{
        if let Some(buf) = find_buffer(idx){
            buf.copy_from(ptr, idx, len);
            let count = min(len, buf.copy_len(idx));
            len -= count;
            unsafe {
                ptr = ptr.add(count);
            }
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
use super::block_device::{sync_read, sync_write};

