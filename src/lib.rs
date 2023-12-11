#![no_std]
#![no_main]

mod gdt;
mod idt;
mod interrupts;
mod io;
mod librs;
mod pic8259;
mod video_graphics_array;

use core::panic::PanicInfo;
use librs::clear;

#[no_mangle]
pub extern "C" fn _start() -> ! {
	init();
	clear();
	println!("Grosse ****");
	println!("****");
	let test1 = 0xabcdef01 as u32;
	let test2 = 0x23456789 as u32;
	let test3 = 0x98765432 as u32;
	let test4 = 0x10fedcba as u32;
	println!("test1: {:x}, test2: {:x}, test3: {:x}, test4: {:x}", test1, test2, test3, test4);
	librs::print_stack();
	loop {
	}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {
	}
}

fn init() {
	gdt::gdt_init();
	idt::idt_init();
	interrupts::pics_init();
	interrupts::enable();
}
