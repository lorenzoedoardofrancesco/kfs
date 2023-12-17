#![no_std]
#![no_main]
#![feature(naked_functions)]

#[macro_use] mod librs;
#[macro_use] mod interrupts;
mod debug;
mod gdt;
mod idt;
mod io;
mod keyboard;
mod memory;
mod pic8259;
mod prompt;
mod shell;
mod video_graphics_array;

use core::arch::asm;
use core::panic::PanicInfo;

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
			0x1D => asm!("int 0x1D", options(nostack)),
			0x1E => asm!("int 0x1E", options(nostack)),
			0x1F => asm!("int 0x1F", options(nostack)),
			0x20 => asm!("int 0x20", options(nostack)),
			0x21 => asm!("int 0x21", options(nostack)),
			0x22 => asm!("int 0x22", options(nostack)),
			0x23 => asm!("int 0x23", options(nostack)),
			_ => panic!("Unsupported interrupt number"),
		}
	}
}

pub fn trigger_divide_by_zero() {
	unsafe {
		asm!(
			"mov eax, 1",   // Load EAX with any non-zero value
			"xor edx, edx", // Clear EDX to ensure clean division
			"div edx",      // Divide EAX by zero (EDX is zero)
			"nop",          // Placeholder instruction (not expected to execute)
			options(noreturn)
		);
	}
}


#[repr(C, align(8))]
struct MultibootHeader {
	magic: u32,
	architecture: u32,
	header_length: u32,
	checksum: u32,
	end_tag_type: u16,
	end_tag_flags: u16,
	end_tag_size: u32,
}

#[used]
#[link_section = ".multiboot_header"]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
	magic: 0xe85250d6,
	architecture: 0,
	header_length: core::mem::size_of::<MultibootHeader>() as u32,
	checksum: (0_u32).wrapping_sub(0xe85250d6).wrapping_sub(0).wrapping_sub(core::mem::size_of::<MultibootHeader>() as u32),
	end_tag_type: 0,
	end_tag_flags: 0,
	end_tag_size: 8,
};

#[repr(C, align(8))]
struct MultibootInfo {
	total_size: u32,
	reserved: u32,
	tags: [MultibootTag; 1],
}

#[repr(C, align(8))]
struct MultibootTag {
	typ: u32,
	size: u32,
}

#[repr(C)]
struct MultibootTagString {
	typ: u32,
	size: u32,
	string: u8,
}

#[repr(C)]
struct MultibootTagModule {
	typ: u32,
	size: u32,
	mod_start: u32,
	mod_end: u32,
	string: u8,
}

#[repr(C)]
struct MultibootTagBasicMemInfo {
	typ: u32,
	size: u32,
	mem_lower: u32,
	mem_upper: u32,
}

#[repr(C)]
struct MultibootTagBootDev {
	typ: u32,
	size: u32,
	biosdev: u32,
	partition: u32,
	sub_partition: u32,
}

#[repr(C)]
struct MultibootMemoryMap {
	typ: u32,
	size: u32,
	entry_size: u32,
	entry_version: u32,
	entries: [MultibootMemoryMapTag; 1],
}

#[repr(C)]
struct MultibootMemoryMapTag {
	size: u32,
	base_addr: u64,
	length: u64,
	typ: u32,
}

const  MULTIBOOT_MEMORY_AVAILABLE: u8 = 1;

#[no_mangle]
pub extern "C" fn _start(multiboot_magic: u32, multiboot_addr: u32) -> ! {
	if multiboot_magic != 0x36d76289 {
		panic!("Invalid multiboot magic number: 0x{:x}", multiboot_magic);
	}
	if multiboot_addr & 0x7 != 0 {
		panic!("Unaligned multiboot address: 0x{:x}", multiboot_addr);
	}
	init();

	let mb_info = unsafe { &*(multiboot_addr as *const MultibootInfo) };
	let mut current_addr = multiboot_addr + 8;

	while current_addr < multiboot_addr + (mb_info.total_size as u32) {
		let tag = unsafe { &*(current_addr as *const MultibootTag) };

		match tag.typ {
			0 => break,  // End tag
			1 => {  // Boot command line
				let cmdline_tag = unsafe { &*(current_addr as *const MultibootTagString) };
				let cmdline = unsafe { core::slice::from_raw_parts((&cmdline_tag.string) as *const u8, cmdline_tag.size as usize - 8) };
				println!("Command line: {}", core::str::from_utf8(cmdline).unwrap());
			},
			2 => {  // Boot loader name
				let loader_tag = unsafe { &*(current_addr as *const MultibootTagString) };
				let loader = unsafe { core::slice::from_raw_parts((&loader_tag.string) as *const u8, loader_tag.size as usize - 8) };
				println!("Boot loader: {}", core::str::from_utf8(loader).unwrap());
			},
			3 => {  // Module
				let module_tag = unsafe { &*(current_addr as *const MultibootTagModule) };
				let module = unsafe { core::slice::from_raw_parts((&module_tag.string) as *const u8, module_tag.size as usize - 8) };
				println!("Module: {}", core::str::from_utf8(module).unwrap());
			},
			4 => {  // Basic memory information
				let mem_tag = unsafe { &*(current_addr as *const MultibootTagBasicMemInfo) };
				println!("Memory: {} KB", mem_tag.mem_lower + mem_tag.mem_upper);
			},
			5 => {  // BIOS boot device
				let bootdev_tag = unsafe { &*(current_addr as *const MultibootTagBootDev) };
				println!("Boot device: 0x{:x}", bootdev_tag.biosdev);
			},
			6 => { // Memory map tag type
				let mmap = unsafe { &*(current_addr as *const MultibootMemoryMap) };
				let entries = (mmap.size as usize - 16) / mmap.entry_size as usize;
	
				let mut entry_addr = current_addr + 16; // Start of the memory map entries
				for _ in 0..entries {
					let entry = unsafe { &*(entry_addr as *const MultibootMemoryMapTag) };
	
					if entry.typ == 1 {
						println!("Available memory region: start = {:x}, length = {:x}", entry.base_addr, entry.length);
					} else {
						println!("Reserved memory region: start = {:x}, length = {:x}", entry.base_addr, entry.length);
					}
	
					entry_addr += mmap.entry_size as u32;
				}}
			// Add other cases for different tag types
			_ => (),
		}

		current_addr = ((current_addr + (tag.size as u32) + 7) & !7) as u32;
	}

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
