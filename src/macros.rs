use crate::debug::DEBUG;
use crate::exceptions::interrupts;
use crate::vga::video_graphics_array::WRITER;
use core::fmt;

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::macros::print(format_args!($($arg)*)));
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
		$crate::macros::print_serial(format_args!($($arg)*))
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

