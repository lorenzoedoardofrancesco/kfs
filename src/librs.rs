use crate::debug::DEBUG;
use crate::exceptions::interrupts;
use crate::vga::video_graphics_array::WRITER;
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

pub fn printk(/*level: &str, */ args: fmt::Arguments) {
	use core::fmt::Write;
	/*let mut writer = WRITER.lock();
	writer.write_str(level).unwrap();
	writer.write_fmt(args).unwrap();*/
	interrupts::disable();
	WRITER.lock().write_fmt(args).unwrap();
	interrupts::enable();
}

///
///
///
use core::arch::asm;

pub fn print_stack() {
	let stack_pointer: usize;
	unsafe {
		asm!("mov {}, esp", out(reg) stack_pointer, options(nomem, nostack));
	}
	printk!("Stack Pointer: {:#8x}\n", stack_pointer);

	const STACK_SIZE: usize = 256; // Define how much of the stack you want to read
	let stack_data =
		unsafe { core::slice::from_raw_parts(stack_pointer as *const u32, STACK_SIZE / 4) };

	for (offset, &value) in stack_data.iter().enumerate() {
		printk!("{:#8x}|{:#8x} ", stack_pointer + offset * 4, value);
		if (offset + 1) % 4 == 0 {
			printk!("\n");
		}
	}
}
