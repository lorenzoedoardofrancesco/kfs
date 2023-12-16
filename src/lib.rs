#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

#[macro_use] mod librs;
#[macro_use] mod interrupts;
mod debug;
mod gdt;
mod idt;
mod io;
mod keyboard;
mod prompt;
mod pic8259;
mod shell;
mod video_graphics_array;

use core::panic::PanicInfo;
use core::arch::asm;

fn generate_interrupt(n: u8) {
    unsafe {
        match n {
            0x00 => asm!("int 0x00", options(nostack)),
            0x01 => asm!("int 0x01", options(nostack)),
            0x02 => asm!("int 0x02", options(nostack)),
			0x03 => asm!("int 0x03", options(nostack)),
			0x04 => asm!("int 0x04", options(nostack)),
			0x05 => asm!("int 0x05", options(nostack)),
			0x06 => asm!("int 0x06", options(nostack)),
			0x07 => asm!("int 0x07", options(nostack)),
			0x08 => asm!("int 0x08", options(nostack)),
			0x09 => asm!("int 0x09", options(nostack)),
			0x0A => asm!("int 0x0A", options(nostack)),
			0x0B => asm!("int 0x0B", options(nostack)),
			0x0C => asm!("int 0x0C", options(nostack)),
			0x0D => asm!("int 0x0D", options(nostack)),
			0x0E => asm!("int 0x0E", options(nostack)),
			0x0F => asm!("int 0x0F", options(nostack)),
			0x10 => asm!("int 0x10", options(nostack)),
			0x11 => asm!("int 0x11", options(nostack)),
			0x12 => asm!("int 0x12", options(nostack)),
			0x13 => asm!("int 0x13", options(nostack)),
			0x14 => asm!("int 0x14", options(nostack)),
			0x15 => asm!("int 0x15", options(nostack)),
			0x16 => asm!("int 0x16", options(nostack)),
			0x17 => asm!("int 0x17", options(nostack)),
			0x18 => asm!("int 0x18", options(nostack)),
			0x19 => asm!("int 0x19", options(nostack)),
			0x1A => asm!("int 0x1A", options(nostack)),
			0x1B => asm!("int 0x1B", options(nostack)),
			0x1C => asm!("int 0x1C", options(nostack)),
            _ => panic!("Unsupported interrupt number"),
        }
    }
}



#[no_mangle]
pub extern "C" fn _start() -> ! {
	init();
	//println!("Grosse ****");
	//println!("****");
	//let test1 = 0xabcdef01 as u32;
	//let test2 = 0x23456789 as u32;
	//let test3 = 0x98765432 as u32;
	//let test4 = 0x10fedcba as u32;
	//println!("test1: {:x}, test2: {:x}, test3: {:x}, test4: {:x}", test1, test2, test3, test4);
	generate_interrupt(0x03);
	loop {
		keyboard::process_keyboard_input();
		librs::hlt();
	}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	loop {
		librs::hlt();
	}
}

fn init() {
	gdt::init();
	idt::init();
	interrupts::init();
	debug::init_serial_port();
	shell::print_welcome_message();
}
