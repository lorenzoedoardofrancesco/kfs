//! # Serial Port Debugging Module
//!
//! Provides functionality for serial port communication, primarily used for debugging purposes.
//! The module defines methods for initializing the serial port and writing data to it. It includes
//! the `Debug` struct that implements the `fmt::Write` trait, allowing formatted strings to be sent
//! over the serial port.

use crate::utils::io::{inb, outb};
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

const SERIAL_PORT: u16 = 0x3f8;

#[allow(dead_code)]
pub enum LogLevel {
	Panic,
	Emergency,
	Alert,
	Critical,
	Error,
	Warning,
	Notice,
	Info,
	Debug,
}

impl LogLevel {
	pub fn as_str(self) -> &'static str {
		match self {
			LogLevel::Panic => "PANIC",
			LogLevel::Emergency => "EMERGENCY",
			LogLevel::Alert => "ALERT",
			LogLevel::Critical => "CRITICAL",
			LogLevel::Error => "ERROR",
			LogLevel::Warning => "WARNING",
			LogLevel::Notice => "NOTICE",
			LogLevel::Info => "INFO",
			LogLevel::Debug => "DEBUG",
		}
	}
}

lazy_static! {
	pub static ref DEBUG: Mutex<Debug> = Mutex::new(Debug {});
}

pub struct Debug;

impl Debug {
	fn is_transmit_empty(&self) -> bool {
		unsafe { (inb(SERIAL_PORT + 5) & 0x20) != 0 }
	}

	fn write_byte_serial(&self, byte: u8) {
		while !self.is_transmit_empty() {}
		unsafe {
			outb(SERIAL_PORT, byte);
		}
	}

	pub fn write_string_serial(&self, s: &str) {
		for byte in s.bytes() {
			self.write_byte_serial(byte);
			if byte == b'\n' {
				self.write_byte_serial(b'\r');
			}
		}
	}
}

impl fmt::Write for Debug {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_string_serial(s);
		Ok(())
	}
}

pub fn init_serial_port() {
	unsafe {
		outb(SERIAL_PORT + 1, 0x00);
		outb(SERIAL_PORT + 3, 0x80);
		outb(SERIAL_PORT + 0, 0x03);
		outb(SERIAL_PORT + 1, 0x00);
		outb(SERIAL_PORT + 3, 0x03);
		outb(SERIAL_PORT + 2, 0xc7);
		outb(SERIAL_PORT + 4, 0x0b);
	}
	log!(LogLevel::Info, "Serial port initialized")
}
