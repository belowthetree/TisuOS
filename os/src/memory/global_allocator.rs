//! # global allocator
//! 用于管理全局的内存分配，分离内核、用户的内存分配
//! 分为内核、用户两个链表管理
//! 2020年12月10日 zg
use core::{mem::size_of, ptr::{null_mut}};
use page::{Page, PageBit};
/// ## Memory
/// 内存管理中最重要的结构
/// 将申请的页表分割成 2 的幂次大小，记录数量和起始地址
/// 每个结构体以链表的形式连接，根据分割粒度的大小排序方便查找
/// 每个颗粒的分配情况通过位图记录，即一位一个颗粒，已用的标记为 1
/// search idx 是辅助搜索的位点，具体可以查看 alloc 和 free 的逻辑
/// Page 记录当前管理内存的属性
#[derive(Copy, Clone)]
pub struct Memory{
    physic_base : *mut u8,
    total_cnt : usize,
    free_cnt : usize,
    use_cnt : usize, // 统计非结构体自身占用块的数量
    size : usize,
    next : *mut Memory,
    bitlen : usize,
    search_idx : usize,
    bitmap : *mut u8,
    page : Page,
}

/// 将某个数向上取 2^n
fn align(x : usize) -> usize{
    let mut rt = 2;
    while rt < x {
        rt *= 2;
    }
    rt
}

#[allow(dead_code)]
impl Memory {
    /// new 负责根据传入 size 申请足够的页表作为堆的分配空间
    /// 它会新建一个 Memory 结构体并插入链表中（排序）
    /// @ size : usize 需要的单位粒度大小
    /// 当创建的粒度较小时，此结构体放在申请页表的开头
    /// 较大则将结构体申请其它内存放置
    pub fn new(size : usize, h : *mut Memory) -> Option<*mut Self> {
        unsafe {
            // 确定对齐大小
            let sz = align(size);
            let num_alloc;
            // 确定分配页数
            num_alloc = Memory::decide_page_num(sz);
            
            let bit_addr;
            let struct_addr;
            let free_cnt;
            let total_size = num_alloc * page::PAGE_SIZE;
            let struct_size = total_size / sz + size_of::<Self>();
            let phy_addr ;
            if (*h).page.is_kernel(){
                phy_addr = page::alloc_kernel_page(num_alloc);
            }
            else{
                phy_addr = page::alloc_user_page(num_alloc);
            }
            // 块的粒度较大时另外存放结构体
            if sz >= MEMORY_BLOCK_A_LITTLE_BIG {
                if (*h).page.is_kernel(){
                    struct_addr = alloc_kernel(struct_size);
                }
                else {
                    struct_addr = alloc_user(struct_size);
                }
                free_cnt = total_size / sz;
            }
            // 如果较小，则直接放置在申请的页表内
            else {
                struct_addr = phy_addr;
                free_cnt = (total_size - struct_size) / sz;
            }
            let prev = Memory::find_first_greater(sz, h);
            bit_addr = struct_addr as usize + size_of::<Self>();
            // 页表申请失败
            if phy_addr.is_null() {
                None
            }
            else {
                let t = struct_addr as *mut Self;
                (*t).init(phy_addr as *mut u8,total_size,
                        sz, bit_addr as *mut u8, free_cnt, &((*h).page));
                (*t).init_bitmap();
                (*prev).append(t);

                Some(t)
            }
        }
    }
    /// 分配以固定大小的块为单位，每次分配需要对齐大小为 2^n
    pub fn alloc(sz : usize, h : *mut Memory) -> *mut u8{
        let size = align(sz);
        unsafe {
            let prev = Memory::find_first_contain(size, h);
            // 没有足够空间，申请新的
            let rt;
            if prev.is_null() {
                match Memory::new(size, h) {
                    Some(m) => rt = m,
                    None => return null_mut()
                }
            }
            else {
                let node = (*prev).next;
                rt = node;
            }
            (*rt).alloc_bitmap()
        }
    }
    /// 根据地址释放，遍历每个结构体，先查看是否包含该地址，然后释放
    /// 如果所有结构体都不包含，则报错
    pub fn free(addr : *mut u8, h : *mut Memory) {
        unsafe {
            let mut head = (*h).next;
            while !head.is_null() && !(*head).is_contain(addr){
                head = (*head).next;
            }
            
            assert!(!head.is_null());

            (*head).free_bitmap(addr);
            let node = &*head;
            // 如果空块太多，释放掉此块
            if node.use_cnt == 0{
                let size = node.size;
                let free_cnt = Memory::get_free_block_num(size, h);
                if free_cnt > 1 && free_cnt * 2 > Memory::get_used_block_num(size, h) {
                    // 如果块结构体在自己管理的页表内
                    if node.free_cnt < node.total_cnt {
                        page::free_page(head as *mut u8);
                    }
                    else {
                        page::free_page(node.physic_base);
                        Memory::free(head as *mut u8, h);
                    }
                    Memory::delete(head);
                }
            }
        }
    }
    /// 获取某个大小的非空块数量
    fn get_used_block_num(size : usize, h : *mut Memory) -> usize {
        unsafe {
            let mut head = (*h).next;
            let mut cnt = 0;
            while !head.is_null() {
                if (*head).size == size && (*head).use_cnt != 0 {
                    cnt += 1;
                }
                head = (*head).next;
            }
            cnt
        }
    }
    /// 获取某个大小的空块数量
    fn get_free_block_num(size : usize, h : *mut Memory) -> usize{
        unsafe {
            let mut head = (*h).next;
            let mut cnt = 0;
            while !head.is_null() {
                if (*head).size == size && (*head).use_cnt == 0 {
                    cnt += 1;
                }
                head = (*head).next;
            }
            cnt
        }
    }
    /// 根据地址找到对应的元素然后释放
    fn free_bitmap(&mut self, addr : *mut u8){
        let st = self.physic_base as usize;
        let idx = (addr as usize - st) / self.size;
        if !self.is_bit_alloc(idx){
            println!("free bitmap addr {:x}", addr as usize);
        }
        assert!(self.is_bit_alloc(idx));
        if idx < self.search_idx {
            self.search_idx = idx;
        }
        unsafe {
            let bit = self.bitmap.add(idx / 8).read_volatile();
            self.bitmap.add(idx / 8).write_volatile(bit & !(1 << (idx % 8)));
            self.free_cnt += 1;
            self.use_cnt -= 1;
        }
        assert!(self.free_cnt <= self.total_cnt);
    }
    /// 元素是否包含此地址
    fn is_contain(&self, addr : *mut u8) -> bool {
        let adr = addr as usize;
        let st = self.physic_base as usize;
        let ed = st + self.total_cnt * self.size;
        adr >= st && adr < ed
    }
    /// 决定某个大小需要的页数
    fn decide_page_num(size : usize) -> usize{
        let too_big = unsafe {MEMORY_BLOCK_A_TOO_BIG};
        let little_big = unsafe {MEMORY_BLOCK_A_LITTLE_BIG};
        if size >= too_big {
            (size + page::PAGE_SIZE - 1) / page::PAGE_SIZE
        }
        else if size >= little_big {
            (size * 4 + page::PAGE_SIZE - 1) / page::PAGE_SIZE
        }
        else {
            1
        }
    }
    /// 初始化变量
    fn init(&mut self, addr : *mut u8, total_size : usize, sz : usize,
            bit_addr : *mut u8, free_cnt : usize, page : &Page) {
            let size = align(sz);
            self.physic_base = addr;
            self.total_cnt = total_size / size;
            self.free_cnt = free_cnt;
            self.size = size;
            self.bitmap = bit_addr;
            self.bitlen = self.total_cnt;
            self.search_idx = self.total_cnt - free_cnt;
            self.use_cnt = 0;
            self.page = page.clone();
        }
    /// 初始化位图，仅当结构体放置在堆所在的页表时调用
    fn init_bitmap(&mut self){
        // println!("size {}, num bit {}, num struct {}", 
        //     self.size, self.bitlen, self.total_cnt - self.free_cnt);
        for i in 0..(self.total_cnt - self.free_cnt) {
            self.write_bitmap(i);
        }
    }
    fn write_bitmap(&mut self, idx : usize){
        unsafe {
            let p = self.bitmap.add(idx / 8);
            p.write_volatile(p.read_volatile() | (1 << (idx % 8)));
        }
    }
    fn append(&mut self, node : *mut Memory){
        unsafe {
            (*node).next = self.next;
            (*self).next = node;
        }
    }
    /// 删除给定的元素，仅从链表中删除，不做其它操作
    fn delete(node : *mut Memory){
        unsafe {
            let mut head;
            if (*node).page.is_kernel(){
                head = (&mut KERNEL_MEMORY) as *mut Memory;
            }
            else {
                head = (&mut USER_MEMORY) as *mut Memory;
            }
            while !(*head).next.is_null() && (*head).next != node {
                head = (*head).next;
            }
            assert!(!(*head).next.is_null());
            (*head).next = (*(*head).next).next;
        }
    }
    /// 找到第一个大于等于 size 的对齐大小的前驱，如果没有则返回最后一个元素
    fn find_first_greater(sz : usize, h : *mut Memory) -> *mut Memory{
        let size = align(sz);
        unsafe {
            let mut head = &mut (*h).next;
            let mut prev = h;
            // 寻找拼接的点位
            while !head.is_null() && (**head).size < size {
                prev = *head;
                head = &mut (**head).next;
            }
            prev
        }
    }
    /// 找到第一个对齐大小相等且能够容纳 size 的元素前驱，找不到则返回空
    fn find_first_contain(size : usize, h : *mut Memory) -> *mut Memory{
        unsafe {
            let mut head = &mut (*h).next;
            let mut prev = h;
            let align_size = align(size);
            // 寻找对齐大小一致并且能够容纳的
            while !head.is_null() && ((**head).size != align_size ||
                    !(**head).can_contain()) {
                
                //println!("in size {}, self size {}, align size {}", 
                //    size, (**head).size, align_size);
                prev = *head;
                head = &mut (**head).next;
            }
            
            if head.is_null(){
                *head
            }
            else {
                prev
            }
        }
    }
    // 检测当前管理单元是否能够找到一块大于等于 size 的内存碎片
    fn can_contain(&self) -> bool {
        self.free_cnt > 0
    }
    // 判断某块内存是否已经分配
    fn is_bit_alloc(&self, idx : usize) -> bool {
        assert!(self.bitlen > idx);
        unsafe{
            self.bitmap.add(idx / 8).read_volatile() & (1 << (idx % 8)) != 0
        }
    }
    fn alloc_bitmap(&mut self) -> *mut u8 {
        for i in self.search_idx..self.bitlen{
            if !self.is_bit_alloc(i) {
                self.write_bitmap(i);
                self.free_cnt -= 1;
                self.use_cnt += 1;
                if self.search_idx == i {
                    self.search_idx += 1;
                }
                return (self.physic_base as usize + i * self.size) as *mut u8;
            }
        }
        null_mut()
    }
    /// 打印内核、用户链表大小及使用情况
    pub fn print(){
        unsafe {
            let mut head = KERNEL_MEMORY.next;
            while !head.is_null() {
                head = (*head).next;
            }
            head = USER_MEMORY.next;
            while !head.is_null() {
                head = (*head).next;
            }
        }
    }
}

pub static mut KERNEL_MEMORY : Memory = Memory{
    physic_base:0 as *mut u8,
    total_cnt : 0,
    free_cnt : 0,
    use_cnt : 0,
    bitlen : 0,
    bitmap : null_mut(),
    size : 0,
    search_idx : 0,
    next : null_mut(),
    page : Page::new(PageBit::Kernel),
};
pub static mut USER_MEMORY : Memory = Memory{
    physic_base:0 as *mut u8,
    total_cnt : 0,
    free_cnt : 0,
    use_cnt : 0,
    bitlen : 0,
    bitmap : null_mut(),
    size : 0,
    search_idx : 0,
    next : null_mut(),
    page : Page::new(PageBit::User),
};
#[allow(dead_code)]
pub static mut KERNEL_HEAP_START : *mut u8 = null_mut();
#[allow(dead_code)]
pub static KERNEL_HEAP_NUM_PAGE : usize = 5;
pub static mut MEMORY_BLOCK_A_LITTLE_BIG : usize= 0;
pub static mut MEMORY_BLOCK_A_TOO_BIG : usize= 0;

/// ## 初始化
pub fn init(){
    unsafe {
        MEMORY_BLOCK_A_LITTLE_BIG = page::PAGE_SIZE / 4;
        MEMORY_BLOCK_A_TOO_BIG = page::PAGE_SIZE;
        KERNEL_MEMORY.page.set_kernel_read_write();
        USER_MEMORY.page.set_user_read_write();
    }
}

/// ## alloc kernel
/// 在内核页表申请内存，暂不清空内存
pub fn alloc_kernel(size : usize) -> *mut u8{
    assert!(size > 0);
    let rt = unsafe {Memory::alloc(size, &mut KERNEL_MEMORY)};
    if rt.is_null(){
        null_mut()
    }
    else {
        unsafe {
            rt.write_bytes(0, size);
        }
        rt
    }
}

/// ## alloc user
/// 在用户页表申请内存，同上
pub fn alloc_user(size : usize) -> *mut u8{
    assert!(size > 0);
    let rt = unsafe {Memory::alloc(size, &mut USER_MEMORY)};
    if rt.is_null(){
        null_mut()
    }
    else {
        unsafe {
            rt.write_bytes(0, size);
        }
        rt
    }
}

pub fn free_user(addr : *mut u8) {
    unsafe {Memory::free(addr, &mut USER_MEMORY)};
}

pub fn free_kernel(addr : *mut u8){
    unsafe {
        Memory::free(addr, &mut KERNEL_MEMORY);
    }
}

/// ## 容器内存管理
/// 实现 RUST 容器的内存分配 trait
/// 所有内存在内核部分分配
struct OSGlobalAlloc;
unsafe impl GlobalAlloc for OSGlobalAlloc {
    unsafe fn alloc(&self, layout : Layout) -> *mut u8{
        alloc_kernel(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free_kernel(ptr);
    }
}

#[global_allocator]
static GA: OSGlobalAlloc = OSGlobalAlloc{};

#[alloc_error_handler]
pub fn alloc_error(layout : Layout) -> !{
    panic!("Fail to alloc {} bytes with {} bytes alignment", layout.size(), layout.align());
}

use core::alloc::{GlobalAlloc, Layout};
use crate::uart;
use crate::page;