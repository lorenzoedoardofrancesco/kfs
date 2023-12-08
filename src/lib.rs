#![no_std]
#![no_main]

mod console;
mod io;
mod librt;

use core::panic::PanicInfo;
use console::{clear_screen, Color};
use crate::librt::putstr;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
	printf!("Hello, {}!\n", "salope");
	printf!("Hello, {}!\n", "bouffe moi le cul");
	printf!("Number: {}\n", "bouffe moi la bite");

    loop {
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}
