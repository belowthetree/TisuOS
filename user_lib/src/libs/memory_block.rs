//! # Block
//! 块内存管理
//! 使用 Rust 的生命周期进行块内存管理，好处在于可以省去内存的回收操作，降低代码复杂度
//! 在所有操作通过接口进行的情况下 Block 是内存安全的（在内存分配器靠谱的情况下），
//! 
//! 2020年12月 zg

use core::{cmp::min, mem::size_of, ptr::slice_from_raw_parts};

use crate::libs::syscall::{free, malloc};

pub struct Block<T>{
    addr : *mut T,
    pub size : usize,
}

#[allow(dead_code)]
impl<T1:Copy> Block<T1> {
    pub fn new(size : usize)->Block<T1>{
        let addr = malloc(size * size_of::<T1>()) as *mut u8 as *mut T1;
        Block {
            addr,
            size,
        }
    }

    pub fn get(&self, idx : usize)->Option<T1>{
        if idx >= self.size{
            None
        }
        else{
            unsafe {
                Some(self.addr.add(idx).read_volatile())
            }
        }
    }

    pub fn set(&self, idx : usize, val : T1, len : usize){
        assert!(idx < self.size);
        unsafe {
            let ptr = self.addr;
            for i in idx..min(idx + len, self.size) {
                ptr.add(i).write_volatile(val);
            }
        }
    }

    /// ## 拷贝
    /// 长度以 other 为准
    pub fn copy_to<T2:Copy>(&self, st1 : usize, other : &Block<T2>, st2 : usize, len : usize){
        assert!(st1 < self.size && st2 < other.size);
        unsafe {
            let ptr1 = self.addr.add(st1) as *mut u8;
            let ptr2 = other.addr.add(st2) as *mut u8;
            let count = min((self.size - st1) * size_of::<T1>(), (other.size - st2) * size_of::<T2>());
            let count = min(len * size_of::<T2>(), count);
            ptr1.copy_to(ptr2, count);
        }
    }

    /// ## 拷贝
    /// 长度以 other 为准
    pub fn copy_from<T2:Copy>(&self, st1 : usize, other : &Block<T2>, st2 : usize, len : usize) {
        assert!(st1 < self.size && st2 < other.size);
        unsafe {
            let ptr1 = self.addr.add(st1) as *mut u8;
            let ptr2 = other.addr.add(st2) as *mut u8;
            let count = min((self.size - st1) * size_of::<T1>(), (other.size - st2) * size_of::<T2>());
            let count = min(len * size_of::<T2>(), count);
            // println!("ptr1 {:x}, ptr2 {:x}, size1 {}, size2 {}, len {}", ptr1 as usize, ptr2 as usize,
                // self.size, other.size, count);
            ptr1.copy_from(ptr2, count);
        }
    }

    pub fn get_addr(&self)->usize {
        self.addr as usize
    }

    pub fn type_as<T>(&self)->&mut T {
        unsafe {
            &mut *(self.addr as *mut T)
        }
    }

    pub fn convert<T2:Copy>(self)->Block<T2> {
        let size = self.size * size_of::<T1>() / size_of::<T2>();
        let rt = Block::<T2>::new(size);
        rt.copy_from(0, &self, 0, self.size);
        rt
    }

    pub fn to_array(&self, st : usize, len : usize)->&'static mut [u8] {
        let len = min(self.size, len) * size_of::<T1>();
        let t = slice_from_raw_parts(
            unsafe {self.addr.add(st)} as *mut u8, len);
        unsafe {&mut *(t as *mut [u8])}
    }

    pub fn array<T>(&self, st : usize, len : usize)->&'static mut [T] {
        let len = min(self.size * size_of::<T1>(), len * size_of::<T>());
        let t = slice_from_raw_parts(
            unsafe {self.addr.add(st)} as *mut T, len / size_of::<T>());
        unsafe {&mut *(t as *mut [T])}
    }
}

impl<T> Drop for Block<T>{
    fn drop(&mut self) {
        free(self.addr as *mut u8 as usize);
    }
}
