//! # 点位图
//! 用于记录使用情况，每一位代表一个块
//! 
//! 2021年1月25日 zg


pub struct Bitmap {
    pub addr : *mut u8,
    pub total_cnt : usize,
    pub free_cnt : usize,
    pub use_cnt : usize, // 统计非结构体自身占用块的数量
    pub search_idx : usize,
}

impl Bitmap {
    /// ### 根据地址找到对应的元素然后释放
    pub fn free(&mut self, idx : usize) {
        assert!(self.is_bit_alloc(idx));
        if idx < self.search_idx {
            self.search_idx = idx;
        }
        unsafe {
            let bit = self.addr.add(idx / 8).read_volatile();
            self.addr.add(idx / 8).write_volatile(bit & !(1 << (idx % 8)));
            self.free_cnt += 1;
            self.use_cnt -= 1;
        }
        assert!(self.free_cnt <= self.total_cnt);
    }
    /// 初始化位图，仅当结构体放置在堆所在的页表时调用
    pub fn init(&mut self, addr : *mut u8, total_cnt : usize, free_cnt : usize, ){
        self.use_cnt = total_cnt - free_cnt;
        self.search_idx = self.use_cnt / 8;
        self.free_cnt = free_cnt;
        self.total_cnt = total_cnt;
        self.addr = addr;
        for i in 0..(self.total_cnt - self.free_cnt) {
            self.write_bitmap(i);
        }
    }

    pub fn alloc_bitmap(&mut self) ->Option<usize> {
        for i in self.search_idx..self.total_cnt{
            if !self.is_bit_alloc(i) {
                self.write_bitmap(i);
                self.free_cnt -= 1;
                self.use_cnt += 1;
                self.search_idx += 1;
                return Some(i);
            }
        }
        None
    }

    fn write_bitmap(&mut self, idx : usize){
        unsafe {
            let p = self.addr.add(idx / 8);
            p.write_volatile(p.read_volatile() | (1 << (idx % 8)));
        }
    }
    // 判断某块内存是否已经分配
    fn is_bit_alloc(&self, idx : usize) -> bool {
        assert!(self.total_cnt > idx);
        unsafe{
            self.addr.add(idx / 8).read_volatile() & (1 << (idx % 8)) != 0
        }
    }
}

// use crate::uart;
