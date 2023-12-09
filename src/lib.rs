#![no_std]
#![no_main]

mod librs;
mod video_graphics_array;
mod io;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
	loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}
