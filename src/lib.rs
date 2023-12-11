#![no_std]
#![no_main]

mod gdt;
mod interrupts;
mod io;
mod librs;
mod pic8259;
mod video_graphics_array;

use core::panic::PanicInfo; 
use librs::clear;

#[no_mangle]
pub extern "C" fn _start() -> ! {
	gdt::gdt_init();
     clear();
    println!("Grosse ****");
    println!("****");
    loop {
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
    }
}

pub fn init() {
    unsafe { interrupts::PICS.lock().initialize() }
	interrupts::enable();
}
