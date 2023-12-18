use core::arch::asm;
use lazy_static::lazy_static;

const CODE_KERNEL_EXECUTE_READ: u8 = 0x9a;	// Code, kernel, execute/read
const CODE_KERNEL_READ: u8 = 0x92;			// Code, kernel, read
const DATA_READ_WRITE: u8 = 0x96;			// Data, kernel, read/write
const CODE_KERNEL_EXECUTE: u8 = 0xfa;		// Code, kernel, execute
const CODE_KERNEL_EXECUTE_READ_CONFORMING: u8 = 0xf2; 	 // Code, kernel, execute/read, conforming
const CODE_KERNEL_EXECUTE_CONFORMING: u8 = 0xf6;		 // Code, kernel, execute, conforming
const GRANULARITY4_KB: u8 = 0xc0;  			// Granularity: 4KB
const SIZE32_BIT: u8 = 0x0f;       			// Default operand and address size: 32-bit
const MAX_SEGMENT_SIZE: u32 = 0xfffff;		// Maximum segment size
const NO_OFFSET: u32 = 0;					// No offset

/// Global Descriptor Table entry structure.
#[repr(C, packed)]
struct GdtEntry {
	limit_low: u16,
	base_low: u16,
	base_middle: u8,
	access: u8,
	granularity: u8,
	base_high: u8,
}

impl GdtEntry {
	/// Creates a new GDT entry.
	fn new(limit: u32, base: u32, access: u8, granularity: u8) -> GdtEntry {
		GdtEntry {
			limit_low: (limit & 0xffff) as u16,
			base_low: (base & 0xffff) as u16,
			base_middle: ((base >> 16) & 0xff) as u8,
			access,
			granularity: (granularity & 0xf0) | (((limit >> 16) & 0x0f) as u8),
			base_high: ((base >> 24) & 0xff) as u8,
		}
	}
}

lazy_static! {
	#[link_section = ".gdt"]
	static ref GDT: [GdtEntry; 7] = [
		GdtEntry::new(0, 0, 0, 0),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, CODE_KERNEL_EXECUTE_READ, GRANULARITY4_KB | SIZE32_BIT),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, CODE_KERNEL_READ, GRANULARITY4_KB | SIZE32_BIT),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, DATA_READ_WRITE, GRANULARITY4_KB | SIZE32_BIT),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, CODE_KERNEL_EXECUTE, GRANULARITY4_KB | SIZE32_BIT),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, CODE_KERNEL_EXECUTE_READ_CONFORMING, GRANULARITY4_KB | SIZE32_BIT),
		GdtEntry::new(MAX_SEGMENT_SIZE, NO_OFFSET, CODE_KERNEL_EXECUTE_CONFORMING, GRANULARITY4_KB | SIZE32_BIT),
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
		"push 0x08",
		"lea eax, [1f]",
		"push eax",
		"retf",
		"1:",
		"mov ax, 0x10",
		"mov ds, ax",
		"mov es, ax",
		"mov fs, ax",
		"mov gs, ax",
		"mov ax, 0x18",
		"mov ss, ax",
		options(preserves_flags)
	);
}

/// Initializes the GDT.
pub fn init() {
	unsafe {
		load_gdt();
		load_segment_registers();
	}
	println_serial!("GDT successfully loaded")
}
