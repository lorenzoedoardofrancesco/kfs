//! # Kernel/Main Entry Point
//!
//! This is the main entry point for the kernel. It includes the initialization of core system components
//! such as GDT, IDT, and handling the primary execution loop. This file sets up the necessary environment
//! for the kernel to function correctly and provides the panic handler for handling system-wide panic conditions.

#![no_std]
#![no_main]
#![feature(naked_functions)]
#[macro_use]
mod macros;
mod boot;
mod exceptions;
mod shell;
mod structures;
mod utils;
mod vga;

use boot::multiboot;
use core::panic::PanicInfo;
use exceptions::{interrupts, keyboard::process_keyboard_input};
use shell::prints;
use structures::{gdt, idt};
use utils::{debug, librs::hlt};

/// The kernel's main entry function.
///
/// This function is called at the start of the kernel and is responsible for initializing
/// various subsystems and entering the main loop.
///
/// # Arguments
///
/// * `multiboot_magic` - The magic number passed by the bootloader.
/// * `multiboot_addr` - The address of the multiboot info structure.
#[no_mangle]
pub extern "C" fn _start(multiboot_magic: u32, multiboot_addr: u32) -> ! {
	init(multiboot_magic, multiboot_addr);
	loop {
		process_keyboard_input();
		hlt();
	}
}

/// Panic handler for the kernel.
///
/// This function is called when a panic occurs anywhere in the kernel.
/// It prints the panic information and halts the system.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {
		hlt();
	}
}

/// Initializes the kernel components.
///
/// Sets up serial port communication for debugging, validates the multiboot header,
/// initializes the GDT, IDT, and interrupts, and displays a welcome message.
fn init(multiboot_magic: u32, multiboot_addr: u32) {
	debug::init_serial_port();
	multiboot::validate_multiboot(multiboot_magic, multiboot_addr);
	gdt::init();
	idt::init();
	interrupts::init();
	prints::print_welcome_message();
}
