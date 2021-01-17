#![no_std]
#![feature(
    asm,
    panic_info_message
)]

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(uart::Uart::new(), $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
	() => ({
		   print!("\r\n")
		   });
	($fmt:expr) => ({
			print!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			print!(concat!($fmt, "\r\n"), $($args)+)
			});
}


#[panic_handler]
fn panic(_info :& PanicInfo) -> ! {
    println!("Panic! ");
    if let Some(p) = _info.location() {
		println!(
				"line {}, file {}: {}",
				p.line(),
				p.file(),
				_info.message().unwrap()
				);
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
            asm!("wfi"::::"volatile");
        }
    }
}

#[no_mangle]
extern "C" fn kernel_init(){
    Uart::new().init();
    trap::init();
    println!("hello kernel");
}

#[no_mangle]
extern "C" fn kernel_start(){

}


use core::panic::PanicInfo;
mod uart;
mod interrupt;
mod cpu;
use uart::Uart;
use interrupt::trap;