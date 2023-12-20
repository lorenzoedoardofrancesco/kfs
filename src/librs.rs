use crate::debug::DEBUG;
use crate::exceptions::interrupts;
use crate::vga::video_graphics_array::WRITER;
use core::arch::asm;
use core::fmt;

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::librs::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
	() => (print!("\n"));
	($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! printk {
	/*($level:expr, $($arg:tt)*) => {
		$crate::librs::printk($level, format_args!($($arg)*))
	};*/

	($($arg:tt)*) => {
		$crate::librs::printk(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! print_serial {
	($($arg:tt)*) => {
		$crate::librs::print_serial(format_args!($($arg)*))
	};

}

macro_rules! println_serial {
	() => (print_serial!("\n\r"));
	($($arg:tt)*) => (print_serial!("{}\n\r", format_args!($($arg)*)));
}

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

pub fn print(args: fmt::Arguments) {
	use core::fmt::Write;
	interrupts::disable();
	WRITER.lock().write_fmt(args).unwrap();
	interrupts::enable();
}

pub fn print_serial(args: fmt::Arguments) {
	use core::fmt::Write;
	interrupts::disable();
	DEBUG.lock().write_fmt(args).unwrap();
	interrupts::enable();
}

//je vais l'ecraser
pub fn printraw(string: &str) {
	interrupts::disable();
	WRITER.lock().write_string_raw(string);
	interrupts::enable();
}

pub fn clear() {
	interrupts::disable();
	WRITER.lock().clear_screen();
	interrupts::enable();
}

#[inline]
pub fn hlt() {
	unsafe {
		asm!("hlt", options(nomem, nostack, preserves_flags));
	}
}

/*/
pub const KERN_EMERG: &str = "KERN_EMERG: ";
pub const KERN_ALERT: &str = "KERN_ALERT: ";
pub const KERN_CRIT: &str = "KERN_CRIT: ";
pub const KERN_ERR: &str = "KERN_ERR: ";
pub const KERN_WARNING: &str = "KERN_WARNING: ";
pub const KERN_NOTICE: &str = "KERN_NOTICE: ";
pub const KERN_INFO: &str = "KERN_INFO: ";
pub const KERN_DEBUG: &str = "KERN_DEBUG: ";
*/

/*
pub fn printk(/*level: &str, */ args: fmt::Arguments) {
	use core::fmt::Write;
	/*let mut writer = WRITER.lock();
	writer.write_str(level).unwrap();
	writer.write_fmt(args).unwrap();*/
	interrupts::disable();
	WRITER.lock().write_fmt(args).unwrap();
	interrupts::enable();
}

*/

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
