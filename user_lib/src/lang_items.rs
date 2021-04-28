// use super::exit;

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let _err = panic_info.message().unwrap();
    if let Some(_location) = panic_info.location() {
        // println!("Panicked at {}:{}, {}", location.file(), location.line(), err);
    } else {
        // println!("Panicked: {}", err);
    }
    loop{}
    // exit(-1);
}