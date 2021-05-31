#![no_std]
#![no_main]

extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    unsafe {
        let ptr = 0 as *mut u8;
        ptr.read_volatile();
    }
}
