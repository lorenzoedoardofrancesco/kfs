//! # Utility Functions for Kernel Operations
//!
//! This module provides various utility functions essential for kernel operations,
//! including string manipulation, reading real-time clock data from CMOS, and performing hex dumps.
//! These functions are for handling shell input, displaying system time, and debugging.

use crate::exceptions::interrupts::TICKS;
use crate::shell::{builtins::MAX_LINE_LENGTH, history::Line};
use crate::utils::io::{inb, outb};
use core::arch::asm;
use core::sync::atomic::Ordering;

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

/// Compares two arrays of bytes.
pub fn array_cmp(a: &Line, b: &Line) -> bool {
	a.iter().zip(b.iter()).all(|(&x, &y)| x == y)
}

/// Converts an array of bytes to a string slice.
pub fn array_to_str(arr: &Line) -> &str {
	let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
	core::str::from_utf8(&arr[..len]).unwrap_or_default()
}

/// Converts a string slice to an array of bytes.
pub fn str_to_array(s: &str) -> Line {
	let mut line = [0; MAX_LINE_LENGTH];
	for (i, c) in s.bytes().enumerate() {
		line[i] = c;
	}
	line
}

/// Converts a Binary-Coded Decimal (BCD) to a binary representation.
pub fn bcd_to_binary(bcd: u8) -> u8 {
	((bcd & 0xf0) >> 4) * 10 + (bcd & 0x0f)
}

/// Reads data from a CMOS register.
///
/// The CMOS is a special chip that stores real-time clock data, such as the current time and date.
/// The CMOS is accessed through I/O ports 0x70 and 0x71. The first port is used to specify the
/// register to read from, and the second port is used to read the data from the register.
///
/// # Safety
///
/// Directly accesses I/O ports, which can cause undefined behavior if misused.
pub fn read_cmos(register: u8) -> u8 {
	unsafe {
		outb(CMOS_ADDRESS, register);
		inb(CMOS_DATA)
	}
}

/// Retrieves the current real-time clock time from CMOS.
pub fn get_rtc_time() -> (u8, u8, u8) {
	let seconds = bcd_to_binary(read_cmos(0x00));
	let minutes = bcd_to_binary(read_cmos(0x02));
	let hours = bcd_to_binary(read_cmos(0x04));

	(hours, minutes, seconds)
}

/// Retrieves the current real-time clock date from CMOS.
pub fn get_rtc_date() -> (u8, u8, u8) {
	let year = bcd_to_binary(read_cmos(0x09));
	let month = bcd_to_binary(read_cmos(0x08));
	let day = bcd_to_binary(read_cmos(0x07));

	(year, month, day)
}

/// Halts the CPU until the next external interrupt.
#[inline]
pub fn hlt() {
	unsafe {
		asm!("hlt", options(nomem, nostack, preserves_flags));
	}
}

pub fn get_tick_count() -> u32 {
	TICKS.load(Ordering::SeqCst)
}

/// Performs a hex dump starting from a given memory address.
pub fn hexdump(mut address: u32, limit: usize) {
	if limit <= 0 {
		return;
	}

	println!("address: {:08x}, limit: {}", address, limit);

	let bytes = unsafe { core::slice::from_raw_parts(address as *const u8, limit) };

	for (i, &byte) in bytes.iter().enumerate() {
		if i % 16 == 0 {
			if i != 0 {
				print_hex_line(address - 16, 16);
				println!();
			}
			print!("{:08x}: ", address);
		}
		print!("{:02x} ", byte);
		address += 1;
	}

	let remaining = limit % 16;
	for _ in 0..((16 - remaining) * 3) {
		print!(" ");
	}
	print_hex_line(address - remaining as u32, remaining);
	println!();
}

/// Helper function for printing a line in hex dump format.
fn print_hex_line(address: u32, count: usize) {
	let bytes = unsafe { core::slice::from_raw_parts(address as *const u8, count) };

	for &byte in bytes {
		if byte <= 32 || byte >= 127 {
			print!(".");
		} else {
			print!("{}", byte as char);
		}
	}
}
