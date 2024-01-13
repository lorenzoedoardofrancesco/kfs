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

use crate::memory::kmalloc::{kfree, kmalloc, kprint_heap, ksize};
use crate::shell::prints;
use boot::multiboot;
use core::panic::PanicInfo;
use exceptions::{interrupts, keyboard::process_keyboard_input, panic::handle_panic};
use structures::{gdt, idt};
use utils::{debug, librs::hlt};
use vga::parrot::animate_parrot;

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
	//unsafe { core::arch::asm!("mov dx, 0; div dx") };
	//crate::memory::kmalloc::kmalloc_tester();
	//PMM.lock().print_memory_map();
	unsafe {
		//memory_management_tester();
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

const HIGH_KERNEL_OFFSET: u32 = 0xC0000000;  //<<<<<< dans const
/// Initializes the kernel components.
///
/// Sets up serial port communication for debugging, validates the multiboot header,
/// initializes the GDT, IDT, and interrupts, and displays a welcome message.
fn init(multiboot_magic: u32, multiboot_addr: u32) {
	multiboot::validate_multiboot(multiboot_magic, multiboot_addr);
	debug::init_serial_port();
	gdt::init();
	idt::init();
	multiboot::read_multiboot_info(multiboot_addr + HIGH_KERNEL_OFFSET);
	//memory::physical_memory_managment::physical_memory_manager_init();
	//memory::page_directory::init_pages();
	interrupts::init();
	prints::print_welcome_message();
}

pub unsafe fn memory_management_tester() {
	// Test 1: Allocate a small block of memory
	println_serial!("\nTest 1: Allocating 256 bytes");
	let address1 = kmalloc(256);
	println_serial!(
		"Address {:?}, Size {:?}",
		address1,
		ksize(address1.unwrap())
	);
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 2: Allocate a larger block of memory
	println_serial!("Test 2: Allocating 1024 bytes");
	let address2 = kmalloc(1024);
	println_serial!(
		"Address {:?}, Size {:?}",
		address2,
		ksize(address2.unwrap())
	);
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 3: Deallocate the first block
	println_serial!("Test 3: Deallocating 256 bytes at {:?}", address1.unwrap());
	kfree(address1.unwrap());
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 4: Allocate another block to see if freed memory is reused
	println_serial!("Test 4: Allocating 256 bytes again");
	let address3 = kmalloc(256);
	println_serial!(
		"Address {:?}, Size {:?}",
		address3,
		ksize(address3.unwrap())
	);
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 5: Allocate a block with a size that requires rounding up to the next page boundary
	println_serial!("Test 5: Allocating 3000 bytes");
	let address4 = kmalloc(3000);
	println_serial!(
		"Address {:?}, Size {:?}",
		address4,
		ksize(address4.unwrap())
	);
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 6: Deallocate the second block
	println_serial!("Test 6: Deallocating 1024 bytes at {:?}", address2.unwrap());
	kfree(address2.unwrap());
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 7: Allocate a very large block of memory
	println_serial!("Test 7: Allocating 377 bytes");
	let address5 = kmalloc(377);
	println_serial!(
		"Address {:?}, Size {:?}",
		address5,
		ksize(address5.unwrap())
	);
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 8: Deallocate all remaining blocks
	println_serial!(
		"Test 8: Deallocating remaining blocks at {:?}, {:?}, {:?}",
		address3.unwrap(),
		address4.unwrap(),
		address5.unwrap()
	);
	kfree(address3.unwrap());
	kfree(address4.unwrap());
	kfree(address5.unwrap());
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 9: Allocate a block of memory that is larger than the entire heap
	println_serial!("Test 9: Allocating 3000 bytes");
	let address6 = kmalloc(3000);
	println_serial!(
		"Address {:?}, Size {:?}",
		address6,
		ksize(address6.unwrap())
	);
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();

	// Test 10: Deallocate the last block
	println_serial!(
		"Test 10: Deallocating 65536 bytes at {:?}",
		address6.unwrap()
	);
	kfree(address6.unwrap());

	println_serial!("Final state of the heap:");
	kprint_heap();
	crate::memory::physical_memory_managment::PMM
		.lock()
		.print_memory_map();
}
