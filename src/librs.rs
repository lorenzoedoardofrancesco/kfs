use core::fmt;

use crate::video_graphics_array::WRITER;

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

	($( $arg:tt )*) => {
		$crate::librs::printk(format_args!($($arg)*))
	};
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn clear() {
    WRITER.lock().clear_screen();
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

pub fn printk(/*level: &str, */args: fmt::Arguments) {
    use core::fmt::Write;
    /*let mut writer = WRITER.lock();
    writer.write_str(level).unwrap();
    writer.write_fmt(args).unwrap();*/
	WRITER.lock().write_fmt(args).unwrap();
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
    printk!("Stack Pointer: {:#X}\n", stack_pointer);

	const STACK_SIZE: usize = 256; // Define how much of the stack you want to read
	let stack_data = unsafe {
		core::slice::from_raw_parts(stack_pointer as *const u32, STACK_SIZE / 4)
	};
	
	for (offset, &value) in stack_data.iter().enumerate() {
		printk!("{:#06x}: {:#010x} ", stack_pointer + offset * 4, value);
		if (offset + 1) % 4 == 0 {
			printk!("\n");
		}
	}
	
}
