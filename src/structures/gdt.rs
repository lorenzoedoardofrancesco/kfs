use crate::structures::accessflags::{
	KERNEL_CODE_SEGMENT, KERNEL_DATA_SEGMENT, KERNEL_STACK_SEGMENT, MAX_SEGMENT_SIZE, NO_OFFSET,
	NULL_SEGMENT, SEGMENT_FLAGS, USER_CODE_SEGMENT, USER_DATA_SEGMENT, USER_STACK_SEGMENT,
};
use core::arch::asm;
use lazy_static::lazy_static;

/// Global Descriptor Table entry structure.
#[repr(C, packed)]
struct GdtEntry {
	limit_low: u16,
	base_low: u16,
	base_middle: u8,
	access: u8,
	flags: u8,
	base_high: u8,
}

impl GdtEntry {
	/// Creates a new GDT entry.
	fn new(limit: u32, base: u32, access: u8, flags: u8, name: &str) -> GdtEntry {
		println_serial!(
			"{:24}{:<#14x}{:<#10x}{:<#11x}{:<#x}",
			name,
			limit,
			base,
			access,
			flags
		);
		GdtEntry {
			limit_low: (limit & 0xffff) as u16,
			base_low: (base & 0xffff) as u16,
			base_middle: ((base >> 16) & 0xff) as u8,
			access,
			flags: (flags & 0xf0) | (((limit >> 16) & 0x0f) as u8),
			base_high: ((base >> 24) & 0xff) as u8,
		}
	}
}

lazy_static! {
	#[link_section = ".gdt"]
	static ref GDT: [GdtEntry; 7] = [
		GdtEntry::new(0, 0, NULL_SEGMENT, 0, "NULL segment"),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, KERNEL_CODE_SEGMENT, SEGMENT_FLAGS, "Kernel code segment"),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, KERNEL_DATA_SEGMENT, SEGMENT_FLAGS, "Kernel data segment"),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, KERNEL_STACK_SEGMENT, SEGMENT_FLAGS, "Kernel stack segment"),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, USER_CODE_SEGMENT, SEGMENT_FLAGS, "User code segment"),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, USER_DATA_SEGMENT, SEGMENT_FLAGS, "User data segment"),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, USER_STACK_SEGMENT, SEGMENT_FLAGS, "User stack segment"),
	];
}

/// Global Descriptor Table register structure.
#[repr(C, packed)]
struct GdtRegister {
	size: u16,
	offset: u32,
}

/// Loads the GDT.
unsafe fn load_gdt() {
	let gdt_register = GdtRegister {
		size: (core::mem::size_of_val(&*GDT) - 1) as u16,
		offset: GDT.as_ptr() as u32,
	};

	asm!("lgdt [{}]", in(reg) &gdt_register, options(readonly, nostack, preserves_flags));
}

/// Loads the segment registers.
unsafe fn load_segment_registers() {
	asm!(
		"push 0x08", // Kernel code segment
		"lea eax, [1f]",
		"push eax",
		"retf",
		"1:",
		"mov ax, 0x10", // Kernel data segment
		"mov ds, ax",
		"mov es, ax",
		"mov fs, ax",
		"mov gs, ax",
		"mov ax, 0x18", // Kernel stack segment
		"mov ss, ax",
		options(preserves_flags)
	);
}

/// Initializes the GDT.
pub fn init() {
	println_serial!("Initializing GDT");
	println_serial!(
		"{:24}{:<14}{:<10}{:<11}{:<}",
		"",
		"limit",
		"offset",
		"access",
		"flags"
	);
	unsafe {
		load_gdt();
		load_segment_registers();
	}
	println_serial!("\n\rGDT successfully loaded")
}
