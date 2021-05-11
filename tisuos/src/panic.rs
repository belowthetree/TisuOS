//! # 错误处理
//! 
//! 2021年4月30日 zg

use core::panic::PanicInfo;
#[lang = "eh_personality"] extern fn eh_personality() {}

#[panic_handler]
fn panic(_info :& PanicInfo) -> ! {
    println!("Panic! ");
    if let Some(p) = _info.location() {
		println!("{:?}: {}", p, _info.message().unwrap());
	}
    else {
        println!("no information");
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> !{
    loop{
        unsafe{
            asm!("wfi");
        }
    }
}