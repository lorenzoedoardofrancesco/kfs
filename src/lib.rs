#![no_std]
#![no_main]
#![feature(naked_functions)]

#[macro_use]
mod librs;
mod boot;
mod exceptions;
mod structures;
mod utils;
mod vga;

use boot::multiboot;
use core::arch::asm;
use core::panic::PanicInfo;
use exceptions::{interrupts, keyboard::process_keyboard_input};
use structures::{gdt, idt};
use utils::{debug, shell};

fn generate_interrupt(n: u8) {
	unsafe {
		match n {
			0x00 => asm!("int 0x00", options(nostack)),
			0x01 => asm!("int 0x01", options(nostack)),
			0x02 => asm!("int 0x02", options(nostack)),
			0x03 => asm!("int 0x03", options(nostack)),
			0x04 => asm!("int 0x04", options(nostack)),
			0x05 => asm!("int 0x05", options(nostack)),
			0x06 => asm!("int 0x06", options(nostack)),
			0x07 => asm!("int 0x07", options(nostack)),
			0x08 => asm!("int 0x08", options(nostack)),
			0x09 => asm!("int 0x09", options(nostack)),
			0x0A => asm!("int 0x0A", options(nostack)),
			0x0B => asm!("int 0x0B", options(nostack)),
			0x0C => asm!("int 0x0C", options(nostack)),
			0x0D => asm!("int 0x0D", options(nostack)),
			0x0E => asm!("int 0x0E", options(nostack)),
			0x0F => asm!("int 0x0F", options(nostack)),
			0x10 => asm!("int 0x10", options(nostack)),
			0x11 => asm!("int 0x11", options(nostack)),
			0x12 => asm!("int 0x12", options(nostack)),
			0x13 => asm!("int 0x13", options(nostack)),
			0x14 => asm!("int 0x14", options(nostack)),
			0x15 => asm!("int 0x15", options(nostack)),
			0x16 => asm!("int 0x16", options(nostack)),
			0x17 => asm!("int 0x17", options(nostack)),
			0x18 => asm!("int 0x18", options(nostack)),
			0x19 => asm!("int 0x19", options(nostack)),
			0x1A => asm!("int 0x1A", options(nostack)),
			0x1B => asm!("int 0x1B", options(nostack)),
			0x1C => asm!("int 0x1C", options(nostack)),
			0x1D => asm!("int 0x1D", options(nostack)),
			0x1E => asm!("int 0x1E", options(nostack)),
			0x1F => asm!("int 0x1F", options(nostack)),
			0x20 => asm!("int 0x20", options(nostack)),
			0x21 => asm!("int 0x21", options(nostack)),
			0x22 => asm!("int 0x22", options(nostack)),
			0x23 => asm!("int 0x23", options(nostack)),
			_ => panic!("Unsupported interrupt number"),
		}
	}
}

pub fn trigger_divide_by_zero() {
	unsafe {
		asm!(
			"mov eax, 1",   // Load EAX with any non-zero value
			"xor edx, edx", // Clear EDX to ensure clean division
			"div edx",      // Divide EAX by zero (EDX is zero)
			"nop",          // Placeholder instruction (not expected to execute)
			options(noreturn)
		);
	}
}

#[no_mangle]
pub extern "C" fn _start(multiboot_magic: u32, multiboot_addr: u32) -> ! {
	init(multiboot_magic, multiboot_addr);
	loop {
		process_keyboard_input();
		librs::hlt();
	}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {
		librs::hlt();
	}
}

fn init(multiboot_magic: u32, multiboot_addr: u32) {
	multiboot::init(multiboot_magic, multiboot_addr);
	gdt::init();
	idt::init();
	interrupts::init();
	shell::print_welcome_message();
}
