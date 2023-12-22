//! # Macros and Printing Utilities
//!
//! Provides macros and utility functions for printing text to the VGA text buffer and serial port.
//! This module is for outputting information to the screen and for debugging purposes.
//! It includes macros for both general printing (`print!` and `println!`) and serial printing
//! (`print_serial!` and `println_serial!`), as well as the implementation for interrupt handlers.

use crate::debug::DEBUG;
use crate::exceptions::interrupts;
use crate::vga::video_graphics_array::{WriteMode, WRITER};
use core::fmt;

/// Macro for printing formatted text to the VGA buffer.
///
/// This macro uses the global `WRITER` instance to output text to the VGA text buffer.
/// It supports variable arguments and formatting, similar to the standard `print!` macro.
#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::macros::print(format_args!($($arg)*)));
}

/// Macro for printing formatted text with a newline to the VGA buffer.
///
/// This macro appends a newline character to the text before printing it.
/// It supports variable arguments and formatting.
#[macro_export]
macro_rules! println {
	() => (print!("\n"));
	($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

/// Macro for printing formatted text to the top of the VGA buffer.
/// 
/// This macro uses the global `WRITER` instance to output text to the top of the VGA text buffer.
#[macro_export]
macro_rules! print_top {
	($($arg:tt)*) => ($crate::macros::print_top(format_args!($($arg)*)));
}

/// Macro for printing formatted text for kernel debugging.
///
/// This macro is specifically designed for kernel-level debugging. It prints
/// text using `printk`, which is typically configured to output to a debugging console.
#[macro_export]
macro_rules! printk {
	($($arg:tt)*) => {
		$crate::librs::printk(format_args!($($arg)*))
	};
}

/// Macro for printing formatted text to the serial port.
///
/// This macro uses the global `DEBUG` instance to output text to the configured serial port.
/// It is typically used for debugging purposes.
#[macro_export]
macro_rules! print_serial {
	($($arg:tt)*) => {
		$crate::macros::print_serial(format_args!($($arg)*))
	};

}

/// Macro for printing formatted text with a newline to the serial port.
///
/// Similar to `println!`, but for serial output. Appends a newline and carriage return.
macro_rules! println_serial {
	() => (print_serial!("\n\r"));
	($($arg:tt)*) => (print_serial!("{}\n\r", format_args!($($arg)*)));
}

/// Macro for creating interrupt handler wrappers.
///
/// Generates a wrapper function for an interrupt handler. This wrapper sets up
/// a proper stack frame, saves and restores registers, and handles interrupt-specific
/// requirements before calling the actual handler function.
#[macro_export]
macro_rules! handler {
	($name: ident) => {{
		#[naked]
		extern "C" fn wrapper() {
			unsafe {
				asm!(
					// Set up stack frame
					"push ebp",
					"mov ebp, esp",

					// Save all general-purpose registers
					"pushad",

					// Calculate the correct stack frame pointer
					"mov eax, esp",
					"add eax, 36", // Adjust for 'pushad' and possibly other pushed registers
					"push eax", // Push stack frame pointer

					// Call the actual interrupt handler
					"call {}",

					// Restore all general-purpose registers
					"pop eax", // Clean up the stack
					"popad",

					// Restore base pointer and return from interrupt
					"pop ebp",
					"iretd",
					sym $name,
					options(noreturn)
				);
			}
		}
		wrapper as extern "C" fn()
	}};
}

/// Prints formatted text to the VGA buffer.
///
/// Disables interrupts, writes formatted text to the VGA buffer, and then re-enables interrupts.
/// This is used by the `print!` macro for actual printing.
pub fn print(args: fmt::Arguments) {
	use core::fmt::Write;
	interrupts::disable();
	let mut writer = WRITER.lock();
	writer.set_mode(WriteMode::Normal);
	writer.write_fmt(args).unwrap();
	interrupts::enable();
}

/// Prints formatted text to the top of the VGA buffer.
/// 
/// Disables interrupts, writes formatted text to the top of the VGA buffer, and then re-enables interrupts.
/// This is used by the `print_top!` macro for actual printing.
pub fn print_top(args: fmt::Arguments) {
	use core::fmt::Write;
	interrupts::disable();
	let mut writer = WRITER.lock();
	writer.set_mode(WriteMode::Top);
	writer.write_fmt(args).unwrap();
	interrupts::enable();
}

/// Prints formatted text to the serial port.
///
/// Similar to `print`, but for serial output. Disables interrupts, writes to the serial port,
/// and then re-enables interrupts. Used by `print_serial!`.
pub fn print_serial(args: fmt::Arguments) {
	use core::fmt::Write;
	interrupts::disable();
	DEBUG.lock().write_fmt(args).unwrap();
	interrupts::enable();
}
