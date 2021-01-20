//! # user allocator
//! 用于处理用户的堆内存申请
//! 采用链式的方法将同一个进程的内存连接起来
//! 2020年12月13日 zg
// This is experimental and requires alloc_prelude as a feature
use core::{mem::size_of, ptr::{null_mut}};
use super::global_allocator;

/// 链接同一个进程的所有堆内存
#[allow(dead_code)]
pub struct MemoryList{
    next : *mut MemoryList,
    phy_addr : *mut u8,
}

#[allow(dead_code)]
impl MemoryList {
    /// 根据大小新建一个，结构体本身存放在内核堆内存里，用户所需在用户内存中申请
    pub fn new(size : usize, is_kernel : bool) -> *mut Self {
        let rt = global_allocator::alloc_kernel(size_of::<Self>()) as *mut Self;
        if rt.is_null() {
            return null_mut();
        }
        unsafe {
            (*rt).next = null_mut();
            if is_kernel {
                (*rt).phy_addr = global_allocator::alloc_kernel(size);
            }
            else {
                (*rt).phy_addr = global_allocator::alloc_user(size);
            }
            if (*rt).phy_addr.is_null() {
                global_allocator::free_kernel(rt as *mut u8);
                null_mut()
            }
            else {
                rt
            }
        }
    }
    /// 从这个节点开始释放，将后续所有节点全部释放，不保证前面的节点被释放
    pub fn free(&mut self, is_kernel : bool){
        let a:*mut Self = self;
        if is_kernel{
            global_allocator::free_kernel(self.phy_addr);
        }
        else {
            global_allocator::free_user(self.phy_addr);
        }
        global_allocator::free_kernel(a as *mut u8);
        if !self.next.is_null() {
            unsafe {
                (*self.next).free(is_kernel);
            }
        }
    }
    /// 申请新的堆块，自动将结构体拼接至末尾
    pub fn alloc(&mut self, size : usize, is_kernel : bool) -> *mut u8{
        let node = Self::new(size, is_kernel);
        if node.is_null() {
            null_mut()
        }
        else {
            let mut h = self;
            // 找到最后一位，添加至末尾
            while !h.next.is_null() {
                h = unsafe{ &mut*(h.next) };
            }
            h.next = node;
            unsafe{
                (*node).phy_addr
            }
        }
    }
}
