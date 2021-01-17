//! # 系统调用封装
//! 2021年1月6日 zg

extern "C" {
    pub fn make_syscall(num : usize, arg1 : usize, arg2 : usize, arg3 : usize, arg4 : usize) -> usize;
}


pub fn fork(func : usize)->usize{
    unsafe {
        make_syscall(57, func, 0, 0, 0)
    }
}

pub fn exec(func : usize, satp : usize){
    unsafe {
        make_syscall(4, func, satp, 0, 0);
    }
}

pub fn list_thread(){
    unsafe {
        make_syscall(5, 0, 0, 0, 0);
    }
}