//! # Keystroke Fusion Surgery
//! ## Rust Kernel for i386 x86 Architecture
//!
//! This project is a minimal operating system kernel for the i386 x86 architecture, written entirely in Rust.
//! The kernel is designed with a focus on simplicity, modularity, and safety, leveraging Rust's powerful
//! features like ownership, zero-cost abstractions, and type safety.
//!
//! ## Overview
//!
//! The kernel includes fundamental components necessary for an operating system such as:
//!
//! - **Boot Process**: Handled by the `boot` module, setting up the environment for the kernel to run.
//! - **Global Descriptor Table (GDT)**: Managed by the `structures::gdt` module, crucial for segment
//!   memory management.
//! - **Interrupt Descriptor Table (IDT)**: Implemented in the `structures::idt` module, managing hardware
//!   and software interrupts.
//! - **Programmable Interrupt Controller (PIC)**: Managed by `exceptions::pic8259`, allowing control over
//!   interrupt signals.
//! - **Interrupt Handling**: Facilitated by `exceptions::interrupts`, providing core interrupt and exception
//!   handling mechanisms.
//! - **Memory Management**: Basic memory management utilities.
//! - **VGA Text Mode Buffer**: For displaying text on the screen, implemented in `vga::video_graphics_array`.
//! - **Shell Interface**: A simple shell interface provided by the `shell` module, for user interaction
//!   and command execution.
//! - **Debugging and Logging**: Tools for debugging and serial communication.
//!
//! ## Running the Kernel
//!
//! The kernel is intended to be run on i386-compatible hardware or emulators (like QEMU).
//! Building and running require a nightly Rust compiler due to the usage of unstable features.
//! The containerized development environment can be used to build and run the kernel.
//! The kernel can be built and run using the following commands:
//! 	make
//!
//! ## Safety and Concurrency
//!
//! While Rust provides many guarantees at compile-time, unsafe code is used for low-level operations,
//! which must be carefully managed. Concurrency in the kernel is minimal but critical, especially in
//! the handling of interrupts.
//!
//! ## MIT License
//!
//! Copyright Keystroke-Fusion-Surgery (c) 2023 Lsimanic-Amuller 42

#![no_std]
#![no_main]
#![feature(naked_functions)]
#[macro_use]
mod macros;
mod boot;
mod exceptions;
mod memory;
mod shell;
mod structures;
mod utils;
mod vga;

use boot::multiboot;
use core::panic::PanicInfo;
use exceptions::{interrupts, keyboard::process_keyboard_input, panic::handle_panic};
use memory::physical_memory_managment::PMM;
use structures::{gdt, idt};
use utils::{debug, librs::hlt};
use vga::parrot::animate_parrot;
use shell::prints;

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
pub extern "C" fn _start(multiboot_magic: usize, multiboot_addr: usize) -> ! {
	init(multiboot_magic, multiboot_addr);
	//unsafe { core::arch::asm!("mov dx, 0; div dx") };
	//crate::memory::kmalloc::kmalloc_tester();
	//PMM.lock().print_memory_map();
	unsafe {
		//memory_management_tester();
		PMM.lock().print_memory_map();
	}
	loop {
		process_keyboard_input();
		animate_parrot();
		hlt();
	}
}

/// Panic handler for the kernel.
///
/// This function is called when a panic occurs anywhere in the kernel.
/// It prints the panic information and halts the system.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	handle_panic(info, None);
}

/// Initializes the kernel components.
///
/// Sets up serial port communication for debugging, validates the multiboot header,
/// initializes the GDT, IDT, and interrupts, and displays a welcome message.
fn init(multiboot_magic: usize, multiboot_addr: usize) {
	//multiboot::validate_multiboot(multiboot_magic, multiboot_addr);
	debug::init_serial_port();
	gdt::init();
	idt::init();
	//multiboot::read_multiboot_info(multiboot_addr);
	//memory::physical_memory_managment::physical_memory_manager_init();
	//memory::page_directory::init_pages();
	interrupts::init();
	prints::print_welcome_message();
}

// pub unsafe fn memory_management_tester() {
// 	// Allocate a small block of memory
// 	let address1 = crate::memory::kmalloc::kmalloc(1024);
// 	println_serial!("Allocated 1024 bytes at {:?}", address1);

// 	// Allocate a larger block of memory
// 	let address2 = crate::memory::kmalloc::kmalloc(4096).unwrap() as *mut usize;
// 	println_serial!("Allocated 4096 bytes at {:?}", address2);

// 	// Deallocate the first block
// 	crate::memory::kmalloc::kfree(address1.unwrap());
// 	println_serial!("Freed memory at {:?}", address1);

// 	// Allocate another block to see if freed memory is reused
// 	let address3 = crate::memory::kmalloc::kmalloc(512);
// 	println_serial!("Allocated 512 bytes at {:?}", address3);

// 	// Additional checks can be performed here...
// 	*address2 = 0xdeadbeef;
// 	let miaomiao = address2.add(0x1000);
// 	miaomiao.write_volatile(0xdeadbeef);

// 	println!("address2: {:#010X}", address2);
// 	println!("miao: {:#010X}", miaomiao);
// }
