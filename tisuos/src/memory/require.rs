//! # 内存管理接口
//! 为内存管理接口的可替换提出接口要求
//! 此要求分为页面、堆内存两种类型
//! 2021年3月22日 zg

/// ## 页面管理需要提供的对外接口
pub trait PageOp {
    /// ### 申请内核用页面
    fn alloc_kernel_page(num : usize)->Option<*mut u8>;
    /// ### 申请用户用页面
    fn alloc_user_page(num : usize)->Option<*mut u8>;
    fn free_kernel_page(addr : *mut u8);
    fn free_user_page(addr : *mut u8);
}

pub trait MemoryOp {
    fn alloc_kernel_memory(size : usize)->Option<*mut u8>;
    fn alloc_user_memory(size : usize)->Option<*mut u8>;
    fn free_kernel_memory(addr : *mut u8);
    fn free_user_memory(addr : *mut u8);
}