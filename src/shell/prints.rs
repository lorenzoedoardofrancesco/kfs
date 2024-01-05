//! # Shell Built-in Command Print Functions
//!
//! This module contains functions to print information to the VGA text buffer in response to
//! shell commands. It includes functions for handling unknown commands, printing a welcome message,
//! displaying a stack trace, and providing a help menu with available commands.

use crate::exceptions::interrupts;
use crate::shell::builtins::clear;
use crate::utils::librs::hexdump;
use crate::vga::prompt;
use crate::vga::video_graphics_array::WRITER;

/// Prints an unknown command error message.
pub fn print_unknown_command(line: &str) {
	let len = line.len().min(50);
	println!("Unknown command: {}", line[0..len].trim());
}

/// Prints the welcome message.
pub fn print_welcome_message() {
	clear();
	println!("                                     :---------:    .---------:---------- ");
	println!("                                   :#@@@@@@@@%=     +@@@@@@@#::@@@@@@@@@@.");
	println!("                                 :#@@@@@@@@%=       +@@@@@%:  :@@@@@@@@@@.");
	println!("                               :#@@@@@@@@%=         +@@@%-    :@@@@@@@@@@.");
	println!("                             :#@@@@@@@@@=           +@%-      :@@@@@@@@@@.");
	println!("                           :#@@@@@@@@@=             =-        -@@@@@@@@@@ ");
	println!("                         :#@@@@@@@@@=                        +@@@@@@@@@*. ");
	println!("                       :#@@@@@@@@@=                        +@@@@@@@@@*.   ");
	println!("                     :#@@@@@@@@@=                        +@@@@@@@@@*.     ");
	println!("                   :#@@@@@@@@@=                        +@@@@@@@@@*.       ");
	println!("                 :#@@@@@@@@@=                        +@@@@@@@@@+.         ");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#        :#.");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#      :#@@.");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#    :#@@@@.");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#  :#@@@@@@.");
	println!("                 ....................=@@@@@@@@@@    +@@@@@@@@@#:#@@@@@@@@.");
	println!("                                     -@@@@@@@@@@     .................... ");
	println!("                                     -@@@@@@@@@@     by                   ");
	println!("                                     -@@@@@@@@@@          Alix Muller     ");
	println!("                                     -@@@@@@@@@@       Lorenzo Simanic    ");
	println!("                                     .----------                          ");
	println!("");
	println!("                       Welcome to KFS! Type 'help' for a list of commands!");
	prompt::init();
}

#[derive(Copy, Clone)]
pub enum PrintStackMode {
	Vga,
	Serial,
}

/// Prints the current stack trace.
///
/// Extracts and displays a hexadecimal dump of the current stack.
/// Useful for debugging purposes.
pub fn print_stack(line: &str, mode: PrintStackMode) {
	use core::ptr;

	let trimmed_line = match mode {
		PrintStackMode::Vga => line["stack".len()..].trim(),
		PrintStackMode::Serial => line["hexdump".len()..].trim(),
	};

	let args = &trimmed_line;

	let mut parts = args.split_whitespace();

	// Determine the address to use for the hex dump
	let address = match parts.next() {
		Some("esp") => {
			let esp: u32;
			unsafe {
				core::arch::asm!("mov {}, esp", out(reg) esp);
			}
			esp
		}
		Some("gdt") => {
			let mut gdtr: [u8; 6] = [0; 6];
			let offset: u32;
			unsafe {
				core::arch::asm!(
					"sgdt [{}]",
					in(reg) &mut gdtr,
					options(nostack, preserves_flags)
				);
				offset = ptr::read_unaligned(gdtr.as_ptr().add(2) as *const u32);
			}
			offset
		}
		Some("idt") => {
			let mut idtr: [u8; 6] = [0; 6];
			let offset: u32;
			unsafe {
				core::arch::asm!(
					"sidt [{}]",
					in(reg) &mut idtr,
					options(nostack, preserves_flags)
				);
				offset = ptr::read_unaligned(idtr.as_ptr().add(2) as *const u32);
			}
			offset
		}
		Some(addr_str) => u32::from_str_radix(addr_str, 16).unwrap_or(0),
		None => 0,
	};

	let num_bytes = parts
		.next()
		.and_then(|arg| arg.parse::<usize>().ok())
		.unwrap_or(256);

	hexdump(address, num_bytes, mode);
}

/// Prints a formatted line in the help menu.
pub fn print_help_line(command: &str, description: &str) {
	print!("  {:13}", command);
	printraw("Z");
	print!("  {:60}", description);
	if command == "shutdown" {
		printraw("Z");
	} else if command != "F12" {
		printraw("ZZ");
	}
}

/// Displays the help menu.
pub fn help() {
	clear();
	printraw("immmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm[Z");
	print!(" Available commands                                                           ");
	printraw("ZlmmmmmmmmmmmmmmmkmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmYZ");
	print_help_line("echo", "display a line of text");
	print_help_line("clear", "clear the screen");
	print_help_line("stack", "print the stack");
	print_help_line("hexdump", "print to the serial COM a hexdump of memory");
	print_help_line("date | time", "display the current date or time");
	print_help_line("miao", "print a cat");
	print_help_line("uname", "print system information");
	print_help_line("halt", "halt the system");
	print_help_line("reboot", "reboot the system");
	print_help_line("shutdown", "shutdown the system");
	printraw("lmmmmmmmmmmmmmmmnmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmYZ");
	print_help_line("F1-F4", "change between screens");
	print_help_line("F8", "parrot stroke activated");
	print_help_line("F9", "display welcome message");
	print_help_line("F10", "change keyboard layout");
	print_help_line("F11", "switch text color");
	print_help_line("F12", "switch background color");

	printraw("ZlmmmmmmmmmmmmmmmjmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmYZ");
	print!(
		" Type 'history' to view command history           {} {} navigate history        ",
		0x1e as char, 0x1f as char
	);
	printraw("Zhmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm\\");
	println!("");
}

/// Prints raw strings to the VGA buffer.
pub fn printraw(string: &str) {
	interrupts::disable();
	WRITER.lock().write_string_raw(string);
	interrupts::enable();
}
