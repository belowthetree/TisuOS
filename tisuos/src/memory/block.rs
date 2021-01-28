//! # Block
//! 块内存管理
//! 使用 Rust 的生命周期进行块内存管理，好处在于可以省去内存的回收操作，降低代码复杂度
//! 在所有操作通过接口进行的情况下 Block 是内存安全的（在内存分配器靠谱的情况下），
//! 
//! 2020年12月 zg

pub struct Block<T : Copy>{
    addr : *mut T,
    pub size : usize,
}

#[allow(dead_code)]
impl<T : Copy> Block<T> {
    pub fn new(size : usize)->Block<T>{
        let addr = alloc(size * size_of::<T>(), true).unwrap() as *mut T;
        Block {
            addr : addr,
            size : size,
        }
    }
    pub fn get(&self, idx : usize)->Option<T>{
        if idx >= self.size{
            None
        }
        else{
            unsafe {
                Some(self.addr.add(idx).read_volatile())
            }
        }
    }
    pub fn set(&mut self, idx : usize, val : T, len : usize){
        assert!(idx < self.size);
        unsafe {
            let ptr = self.addr;
            for i in idx..min(len, self.size) {
                ptr.add(i).write_volatile(val);
            }
        }
    }

    pub fn copy_to(&self, other : &Block<T>, len : usize){
        unsafe {
            let ptr = other.addr;
            let count = min(self.size, other.size);
            let count = min(len, count);
            self.addr.copy_to(ptr, count);
        }
    }
    
    pub fn copy_from(&self, other : &Block<T>, len : usize) {
        unsafe {
            let ptr = other.addr;
            let count = min(self.size, other.size);
            let count = min(len, count);
            other.addr.copy_to(ptr, count);
        }
    }
    
}

impl<T : Copy> Drop for Block<T>{
    fn drop(&mut self) {
        free(self.addr as *mut u8);
    }
}

use core::{cmp::min, mem::size_of};
use super::allocator::{alloc, free};