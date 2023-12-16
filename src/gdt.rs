use core::arch::asm;
use lazy_static::lazy_static;

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
		GdtEntry::new(0xfffff, 0, 0x9a, 0xcf),
		GdtEntry::new(0xfffff, 0, 0x92, 0xcf),
		GdtEntry::new(0xfffff, 0, 0x96, 0xcf),
		GdtEntry::new(0xfffff, 0, 0xfa, 0xcf),
		GdtEntry::new(0xfffff, 0, 0xf2, 0xcf),
		GdtEntry::new(0xfffff, 0, 0xf6, 0xcf),
	];
}

#[repr(C, packed)]
struct GdtRegister {
	size: u16,
	offset: u32,
}

unsafe fn load_gdt() {
	let gdt_register = GdtRegister {
		size: (core::mem::size_of_val(&*GDT) - 1) as u16,
		offset: GDT.as_ptr() as u32,
	};

	asm!("lgdt [{}]", in(reg) &gdt_register, options(readonly, nostack, preserves_flags));
}

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

pub fn init() {
	unsafe {
		load_gdt();
		load_segment_registers();
	}
}
