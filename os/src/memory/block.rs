//! # Block
//! 块内存管理
//! 使用 Rust 的 Box 容器进行块内存管理，好处在于可以省去内存的回收操作，降低代码复杂度
//! 2020年12月 zg
use core::cmp::min;

use alloc::{prelude::v1::*};
pub struct Block{
    pub addr : *mut u8,
    pub size : usize,
}
impl Block {
    /// 获取内存中的某个值
    #[allow(dead_code)]
    pub fn get(&self, idx : usize)->Option<u8>{
        if idx >= self.size{
            None
        }
        else{
            unsafe {
                Some(self.addr.add(idx).read_volatile())
            }
        }
    }
    #[allow(dead_code)]
    pub fn set(&mut self, idx : usize, val : u8){
        assert!(idx < self.size);
        unsafe {
            self.addr.add(idx).write_volatile(val);
        }
    }
    pub fn copy_to(&self, target : *mut u8, len : usize){
        unsafe {
            target.copy_from(self.addr, min(len, self.size));
        }
    }
}

pub fn new_block(size : usize) -> Box::<Block>{
    let b = Block{
        addr : alloc_kernel(size),
        size : size,
    };
    assert!(!b.addr.is_null());
    Box::new(b)
}

impl Drop for Block{
    fn drop(&mut self) {
        free_kernel(self.addr);
    }
}

use super::global_allocator::{alloc_kernel, free_kernel};