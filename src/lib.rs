#![no_std]
#![no_main]
#![feature(naked_functions)]
#[macro_use]
mod macros;
mod boot;
mod exceptions;
mod structures;
mod utils;
mod vga;
mod shell;

use boot::multiboot;
use core::panic::PanicInfo;
use exceptions::{interrupts, keyboard::process_keyboard_input};
use structures::{gdt, idt};
use utils::{ debug, librs::hlt };
use shell::prints;


#[no_mangle]
pub extern "C" fn _start(multiboot_magic: u32, multiboot_addr: u32) -> ! {
	init(multiboot_magic, multiboot_addr);
	loop {
		process_keyboard_input();
		hlt();
	}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {
		hlt();
	}
}

fn init(multiboot_magic: u32, multiboot_addr: u32) {
	debug::init_serial_port();
	multiboot::validate_multiboot(multiboot_magic, multiboot_addr);
	gdt::init();
	idt::init();
	interrupts::init();
	prints::print_welcome_message();
}
