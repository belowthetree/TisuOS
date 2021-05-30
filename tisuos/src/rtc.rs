//! # 实时时间
//! 仅支持 QEMU 5.0 以上的稳定版
//!
//! 2021年5月13日 zg

pub const BASE_ADDR     : usize = 0x101000;
pub const START_YEAR    : usize = 1970;

pub fn read()->usize {
    unsafe {
        let ptr = BASE_ADDR as *const u32;
        let mut time = ptr.read_volatile() as usize;
        time |= (ptr.add(1).read_volatile() as usize) << 32;
        time
    }
}

#[derive(Debug)]
pub struct Time {
    pub year : usize,
    pub month : usize,
    pub day : usize,
    pub hour : usize,
    pub minute : usize,
    pub second : usize,
}

impl Time {
    /// 精确到秒
    pub fn from_timestamp(tm : usize)->Self {
        let second = tm % 60;
        let minute = tm / 60 % 60;
        let hour = (tm / 3600 + 8) % 24;
        let total_day = (tm / 3600 + 8) / 24;
        let mut year = START_YEAR;
        let mut re_day = total_day;
        while re_day > 366 {
            let b = year % 100 != 0;
            if b && year % 4 == 0 || b && year % 400 == 0 {
                re_day -= 366;
            }
            else {
                re_day -= 365;
            }
            year += 1;
        }
        let b = year % 100 != 0;
        if re_day == 365 && b && year % 4 != 0 && b && year % 400 != 0 {
            year += 1;
            re_day -= 365;
        }
        let mut month = 1;
        while re_day > 31 {
            if month <= 7 {
                if month == 2 {
                    if year % 100 != 0 && year % 4 == 0 || year % 100 == 0 && year % 400 == 0 {
                        re_day -= 29;
                    }
                    else {
                        re_day -= 28;
                    }
                }
                else if month % 2 == 0 {
                    re_day -= 30;
                }
                else {
                    re_day -= 31;
                }
            }
            else {
                if month % 2 == 0 {
                    re_day -= 31;
                }
                else {
                    re_day -= 30;
                }
            }
            month += 1;
        }

        Self {
            year,
            month,
            day : re_day,
            hour,
            minute,
            second,
        }
    }

    pub fn read()->Self {
        Self::from_timestamp(read() / 10_0000_0000)
    }
}