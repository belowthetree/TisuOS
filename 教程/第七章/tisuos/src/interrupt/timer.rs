//! timer
//! 控制 timer 触发
//! 2020年12月12日 zg
static FREQUENCY    : usize = 1000_0000;
static MTIMECMP     : usize = 0x200_4000;
static MTIME        : usize = 0x200_BFF8;
static INTERVAL     : u64 = 15_0000;

/// 设置下一个时间中断的间隔，单位是秒
#[allow(dead_code)]
pub fn set_next_interrupt(seconds : usize){
    unsafe {
        let mtimecmp = MTIMECMP as *mut u64;
        let mtime = MTIME as *mut u64;
        mtimecmp.write_volatile(mtime.read_volatile() + (seconds * FREQUENCY) as u64);
    }
}
#[allow(dead_code)]
/// 获取当前计时器的计数
pub fn get_time() -> u64{
    unsafe {
        let mtime = MTIME as *mut u64;
        mtime.read_volatile()
    }
}

pub fn get_million_time()->usize{
    unsafe {
        let mtime = MTIME as *mut u64;
        mtime.read_volatile() as usize / (FREQUENCY / 1000)
    }
}
/// 设置下一个 timer 触发，使用默认 INTERVAL
pub fn set_next_timer() {
    unsafe {
        let mtimecmp = MTIMECMP as *mut u64;
        let mtime = MTIME as *mut u64;
        mtimecmp.write_volatile(mtime.read_volatile() + INTERVAL as u64);
    }
}

#[allow(dead_code)]
pub fn get_timecmp() -> u64{
    unsafe {
        let mtimecmp = MTIMECMP as *mut u64;
        mtimecmp.read_volatile()
    }
}
