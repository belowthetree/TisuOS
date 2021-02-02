//! # 内存分配器
//! 将内存按照 2 的幂次对齐后进行分配
//! 分为内核、用户两种内存
//! 
//! 2021年1月25日 zg

#![allow(dead_code)]

/// 将某个数向上取 2^n
fn align(x : usize) -> usize{
    let mut rt = 2;
    while rt < x {
        rt *= 2;
    }
    rt
}

const MEMORY_TOO_BIG : usize = PAGE_SIZE;
const MEMORY_SIZE_INSIDE : usize = PAGE_SIZE / 20;

pub struct Memory {
    physic_base : *mut u8,
    size : usize,
    next : Option<*mut Memory>,
    bitmap : Bitmap,
    is_kernel : bool,
}
/// ## 对外接口
impl Memory {
    /// ### 生成空 Memory，用于作为链表头部
    pub const fn empty(is_kernel : bool)->Self {
        Self {
            physic_base : null_mut(),
            size : 0,
            next : None,
            bitmap : Bitmap::empty(),
            is_kernel : is_kernel,
        }
    }
    /// 分配以固定大小的块为单位，每次分配需要对齐大小为 2^n
    pub fn alloc(&mut self, size : usize) ->Option<*mut u8> {
        let size = align(size);
        unsafe {
            let node = self.find_first_contain(size);
            let rt;
            if let Some(node) = node {
                rt = node;
            }
            // 没有足够空间，申请新的
            else {
                match self.new(size) {
                    Some(m) => rt = m,
                    None => return None
                }
            }
            if let Some(idx) = (*rt).bitmap.alloc_bitmap() {
                Some((idx * (*rt).size + (*rt).physic_base as usize) as *mut u8)
            }
            else {
                None
            }
        }
    }
    /// ### 根据地址释放，遍历每个结构体，先查看是否包含该地址，然后释放
    /// 如果所有结构体都不包含，则报错
    pub fn free(&mut self, addr : *mut u8) {
        unsafe {
            let mut head = self.next;

            while !(*head.unwrap()).is_contain(addr){
                head = (*head.unwrap()).next;
            }

            (*head.unwrap()).free_bitmap(addr);
            let node = &mut *head.unwrap();
            // 如果空块太多，释放掉此块
            if node.bitmap.use_cnt == 0 {
                let size = node.size;
                let free_cnt = self.get_free_block_num(size);
                let use_cnt = self.get_used_block_num(size);
                if free_cnt > 1 && free_cnt * 2 > use_cnt {
                    // 如果块结构体在自己管理的页表内
                    if node.is_inside() {
                        page::free_page(head.unwrap() as *mut u8);
                    }
                    else {
                        page::free_page(node.physic_base);
                        self.free(head.unwrap() as *mut u8);
                    }
                    self.delete(head.unwrap());
                }
            }
        }
    }
    /// ### 调试输出
    pub fn print(&self) {
        let mut head = self.next;
        unsafe {
            while head.is_some() {
                let h = head.unwrap();
                println!("size {}, total {}, used {}, free {}", (*h).size, (*h).bitmap.total_cnt,
                    (*h).bitmap.use_cnt, (*h).bitmap.free_cnt);
                head = (*head.unwrap()).next;
            }
        }
    }
    
}

/// ## 私有辅助方法
impl Memory {
    /// ### new 负责根据传入 size 申请足够的页表作为堆的分配空间
    /// 它会新建一个 Memory 结构体并插入链表中（排序）
    /// @ size : usize 需要的单位粒度大小
    /// 当创建的粒度较小时，此结构体放在申请页表的开头
    /// 较大则将结构体申请其它内存放置
    fn new(&mut self, size : usize) -> Option<*mut Self> {
        // 确定对齐大小
        let sz = align(size);
        let num_alloc;
        // 确定分配页数
        num_alloc = self.decide_page_num(sz);
        
        let bit_addr;
        let struct_addr;
        let free_cnt;
        let total_size = num_alloc * page::PAGE_SIZE;
        let struct_size = total_size / sz + size_of::<Self>();
        let phy_addr ;
        if self.is_kernel {
            phy_addr = page::alloc_kernel_page(num_alloc);
        }
        else{
            phy_addr = page::alloc_user_page(num_alloc);
        }
        // 块的粒度较大时另外存放结构体
        if sz >= MEMORY_SIZE_INSIDE {
            if self.is_kernel {
                if let Some(addr) = alloc_kernel(struct_size) {
                    struct_addr = addr;
                }
                else {
                    return None;
                }
            }
            else {
                if let Some(addr) = alloc_user(struct_size) {
                    struct_addr = addr;
                }
                else {
                    return None;
                }
            }
            free_cnt = total_size / sz;
        }
        // 如果较小，则直接放置在申请的页表内
        else {
            struct_addr = phy_addr;
            free_cnt = (total_size - struct_size) / sz;
        }
        bit_addr = struct_addr as usize + size_of::<Self>();
        // 页表申请失败
        if phy_addr.is_null() {
            None
        }
        else {
            let t = struct_addr as *mut Self;
            unsafe {
                (*t).init(phy_addr as *mut u8,total_size,
                    sz, bit_addr as *mut u8, free_cnt, self.is_kernel);
            self.append(t);
            }
            Some(t)
        }
    }
    /// ### 初始化变量
    fn init(&mut self, addr : *mut u8, total_size : usize, sz : usize,
        bit_addr : *mut u8, free_cnt : usize, is_kernel : bool) {
        let size = align(sz);
        self.physic_base = addr;
        let total_cnt = total_size / size;
        self.bitmap.init(bit_addr, total_cnt, free_cnt);
        self.size = size;
        // self.bitlen = self.total_cnt / 8;
        self.is_kernel = is_kernel;
    }
    fn is_inside(&self)->bool {
        self.size < MEMORY_TOO_BIG
    }
    /// ### 获取某个大小的非空块数量
    fn get_used_block_num(&self, size : usize) ->usize {
        unsafe {
            let mut head = self.next;
            let mut cnt = 0;
            while head.is_some() {
                if (*head.unwrap()).size == size && !(*head.unwrap()).bitmap.use_cnt == 0 {
                    cnt += 1;
                }
                else if (*head.unwrap()).size != size {
                    break;
                }
                head = (*head.unwrap()).next;
            }
            cnt
        }
    }
    /// ### 获取某个大小的空块数量
    fn get_free_block_num(&self, size : usize) ->usize {
        unsafe {
            let mut head = self.next;
            let mut cnt = 0;
            while head.is_some() {
                if (*head.unwrap()).size == size && (*head.unwrap()).bitmap.use_cnt == 0 {
                    cnt += 1;
                }
                else if (*head.unwrap()).size > size{
                    break;
                }
                head = (*head.unwrap()).next;
            }
            cnt
        }
    }
    /// ### 根据地址找到对应的元素然后释放
    fn free_bitmap(&mut self, addr : *mut u8){
        let st = self.physic_base as usize;
        let idx = (addr as usize - st) / self.size;
        self.bitmap.free(idx);
    }
    /// 元素是否包含此地址
    fn is_contain(&self, addr : *mut u8) -> bool {
        let adr = addr as usize;
        let st = self.physic_base as usize;
        let ed = st + self.bitmap.total_cnt * self.size;
        adr >= st && adr < ed
    }
    /// ### 决定某个大小需要的页数
    fn decide_page_num(&self, size : usize) -> usize{
        let too_big = MEMORY_TOO_BIG;
        if size > too_big {
            (size + page::PAGE_SIZE - 1) / page::PAGE_SIZE
        }
        else {
            (size * 4 + page::PAGE_SIZE - 1) / page::PAGE_SIZE
        }
    }
    /// ### 添加至第一个 size 小于等于 node 的节点前方
    fn append(&mut self, node : *mut Memory){
        unsafe {
            let mut head = Some(self as *mut Memory);
            let size = (*node).size;
            while head.is_some() && (*head.unwrap()).next.is_some() && (*(*head.unwrap()).next.unwrap()).size < size {
                head = (*head.unwrap()).next;
            }
            
            (*node).next = (*head.unwrap()).next;
            (*head.unwrap()).next = Some(node);
        }
    }
    /// ### 删除给定的元素，仅从链表中删除，不做其它操作
    fn delete(&mut self, node : *mut Memory){
        unsafe {
            let mut head = Some(self as *mut Memory);
            while (*head.unwrap()).next.unwrap() != node {
                head = (*head.unwrap()).next;
            }
            
            (*head.unwrap()).next = (*(*head.unwrap()).next.unwrap()).next;
        }
    }
    /// ### 找到第一个对齐大小相等且能够容纳 size 的元素前驱，找不到则返回空
    fn find_first_contain(&self, size : usize) ->Option<*mut Memory>{
        unsafe {
            let mut head = self.next;
            // 寻找对齐大小一致并且能够容纳的
            while head.is_some() && ((*head.unwrap()).size != size || (*head.unwrap()).bitmap.free_cnt == 0) {
                head = (*head.unwrap()).next;
            }
            
            head
        }
    }
}

pub static mut KERNEL_ALLOCATOR : Memory = Memory::empty(true);
static mut USER_ALLOCATOR : Memory = Memory::empty(false);
static mut MEMORY_LOCK : Mutex = Mutex::new(); // 内存控制同步锁

/// ## 外部内存申请接口
pub fn alloc(size : usize, is_kernel : bool)->Option<*mut u8> {
    unsafe {
        MEMORY_LOCK.lock();
    }
    let rt;
    if is_kernel{
        rt = alloc_kernel(size);
    }
    else {
        rt = alloc_user(size);
    }
    unsafe {
        MEMORY_LOCK.unlock();
    }
    rt
}

/// ## 外部内存释放接口
pub fn free(addr : *mut u8){
    unsafe {
        MEMORY_LOCK.lock();
    }
    if addr as usize >= unsafe {page::KERNEL_HEAP_START} && unsafe {page::USER_HEAP_START} > addr as usize {
        free_kernel(addr);
    }
    else if unsafe {page::USER_HEAP_START} <= addr as usize {
        free_user(addr);
    }
    else {
        panic!("free addr err {:x}", addr as usize);
    }
    unsafe {
        MEMORY_LOCK.unlock();
    }
}


/// ## 分配内核内存
/// 在内核页表申请内存，暂不清空内存
fn alloc_kernel(size : usize) ->Option<*mut u8> {
    assert!(size > 0);
    let rt = unsafe {KERNEL_ALLOCATOR.alloc(size)};
    if rt.is_some(){
        let rt = rt.unwrap();
        unsafe {
            rt.write_bytes(0, size);
        }
        if unsafe {page::KERNEL_HEAP_START} > rt as usize || unsafe {page::USER_HEAP_START} <= rt as usize {
            panic!("out memory {:x} {:x}", unsafe {page::KERNEL_HEAP_START}, rt as usize);
        }
        Some(rt)
    }
    else {
        None
    }
}

/// ## 分配用户内存
/// 在用户页表申请内存，同上
fn alloc_user(size : usize) ->Option<*mut u8> {
    assert!(size > 0);
    let rt = unsafe {USER_ALLOCATOR.alloc(size)};
    if rt.is_some(){
        let rt = rt.unwrap();
        unsafe {
            rt.write_bytes(0, size);
        }
        if unsafe {page::USER_HEAP_START} > rt as usize || unsafe {MEMORY_END} <= rt as usize {
            panic!("out memory {:x} {:x}", unsafe {page::KERNEL_HEAP_START}, rt as usize);
        }
        Some(rt)
    }
    else {
        None
    }
}

fn free_kernel(addr : *mut u8) {
    unsafe {
        KERNEL_ALLOCATOR.free(addr);
    }
}

fn free_user(addr : *mut u8) {
    unsafe {
        USER_ALLOCATOR.free(addr);
    }
}

pub fn print() {
    unsafe {
        KERNEL_ALLOCATOR.print();
    }
}

pub fn test(){
    println!("test global_allocator alloc");

    {
        let mut s = String::new();
        for i in 0..500 {
            s.push('a');
            if i % 100 == 0 {
                s.push('.');
            } 
        }
        let _ : Vec::<&str> = s.split('.').collect();
        while s.len() > 0 {
            s.pop();
        }
    }

    for i in 0..50 {
        alloc(i + 1, true);
        // println!("alloc addr {:x}", alloc(i + 1, true).unwrap() as usize);
    }
    // print();
    // for _ in 0..10 {
    //     alloc(4096, true);
    //     // println!("alloc addr {:x}", alloc(4096, true).unwrap() as usize);
    // }
    print();
    let a1 = alloc(4096, true).unwrap();
    let a2 = alloc(4096, true).unwrap();
    let a3 = alloc(4096, true).unwrap();
    let a4 = alloc(4096, true).unwrap();

    let b1 = alloc(4096, true).unwrap();
    let b2 = alloc(4096, true).unwrap();
    let b3 = alloc(4096, true).unwrap();
    let b4 = alloc(4096, true).unwrap();

    println!("before delete");
    page::print();
    print();

    free(a1);
    free(a2);

    free(b1);
    free(b2);

    println!("after delete");
    page::print();
    print();
    free(b3);
    free(b4);
    free(a3);
    free(a4);

    println!("after delete");
    page::print();
    print();

    println!("test kernel container");

    {
        println!("before container");
        page::print();
        print();
        let b = Box::new(10);
        println!("alloc box {}", *b);
        let v = vec![1, 3, 5];
        println!("vec values: ");
        for i in v{
            print!("{} ", i);
        }
        println!();
        println!("after container");
        page::print();
        print();
    }

    let mut v = Vec::<i32>::new();
    for i in 0..10 {
        v.push(i);
    }
    for i in v{
        print!("{}", i);
    }
    println!();
}

/// ## 容器内存管理
/// 实现 RUST 容器的内存分配 trait
/// 所有内存在内核部分分配
struct OSGlobalAlloc;
unsafe impl GlobalAlloc for OSGlobalAlloc {
    unsafe fn alloc(&self, layout : Layout) -> *mut u8{
        alloc(layout.size(), true).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr);
    }
}

#[global_allocator]
static GA: OSGlobalAlloc = OSGlobalAlloc{};

#[alloc_error_handler]
pub fn alloc_error(layout : Layout) -> !{
    panic!("Fail to alloc {} bytes with {} bytes alignment", layout.size(), layout.align());
}

use alloc::prelude::v1::*;
use core::{alloc::{GlobalAlloc, Layout}, mem::size_of, ptr::null_mut};
use crate::{sync::Mutex, uart};
use super::{MEMORY_END, bitmap::{Bitmap}, page::{self, PAGE_SIZE}};
