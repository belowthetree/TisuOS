#![no_std]

#[panic_handler]
fn panic(_info :& PanicInfo) -> ! {
    if let Some(p) = _info.location() {
	}
    else {
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> !{
    loop{
    }
}

use core::panic::PanicInfo;

