use core::fmt;

use crate::video_graphics_array::WRITER;

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::librs::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
	() => (print!("\n"));
	($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	WRITER.lock().write_fmt(args).unwrap();
}