use lazy_static::lazy_static;
use core::arch::asm;

/* GDT Entry Structure | 64 bits
Base Address: This is the starting address of the memory segment.
			  In i386, this is a 32-bit value split across the entry.
			  En gros pointeur sur le début du segment. Donc 32 bits.
Limit: This specifies the size of the segment. 16 bits ici 0xYYYY
	   The limit can be up to 4GB (when using granularity). Regarder en bas.
Access Byte: This contains flags that define the type of segment (code or data),
			 the privilege level (0 for kernel, 3 for user),
			 and other access rights (like read/write/execute permissions).
Granularity: Bits 0-3 (4 bits): Higher bits of the segment limit. These, combined with the 16 bits from limit_low, form a 20-bit limit value.
			 Bit 4 (1 bit): Specifies the size of the operation (0 for 16-bit, 1 for 32-bit).
			 Bit 5 (1 bit): Always set to 1 in a valid data or code segment.
			 Bit 6 (1 bit): Available for system use (often unused).
			 Bit 7 (Granularity Bit, 1 bit):
			 This bit, if set, means the segment limit is in 4KB blocks
			 En gros tu veux 1 byte : Limit = 0x00001 et Granularity = 0
			 Tu veux 1MB : Limit = 0xFFFFF et Granularity = 0
			 Tu veux 4KB : Limit = 0x00001 et Granularity = 1 or 0x01000 et Granularity = 0
			 Tu veux 4GB : Limit = 0xFFFFF et Granularity = 1 */

#[repr(C, packed)]
struct GDT_Entry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GDT_Entry {
    fn new(limit: u32, base: u32, access: u8, granularity: u8) -> GDT_Entry {
        GDT_Entry {
            limit_low: (limit & 0xffff) as u16,
            base_low: (base & 0xffff) as u16,
            base_middle: ((base >> 16) & 0xff) as u8,
            access,
            granularity: (granularity & 0xf0) | (((limit >> 16) & 0x0f) as u8),
            base_high: ((base >> 24) & 0xff) as u8,
        }
    }
}

// https://wiki.osdev.org/GDT_Tutorial#Flat_.2F_Long_Mode_Setup
// https://www.independent-software.com/operating-system-development-protected-mode-global-descriptor-table.html
// http://www.osdever.net/bkerndev/Docs/gdt.htm

lazy_static! {
    static ref GDT: [GDT_Entry; 7] = [
        GDT_Entry::new(0, 0, 0, 0), // null descriptor segment
        GDT_Entry::new(0xfffff, 0, 0x9a, 0xcf), // kernel mode code segment
        GDT_Entry::new(0xfffff, 0, 0x92, 0xcf), // kernel mode data segment
        GDT_Entry::new(0xfffff, 0, 0x96, 0xcf), // kernel mode stack segment
        GDT_Entry::new(0xfffff, 0, 0xfa, 0xcf), // user mode code segment
        GDT_Entry::new(0xfffff, 0, 0xf2, 0xcf), // user mode data segment
        GDT_Entry::new(0xfffff, 0, 0xf6, 0xcf), // user mode stack segment
    ];
}

// Global Descriptor Table Register | 48 bits
// Limit: A 16-bit value specifying the size of the GDT in bytes minus one. This is because the limit is zero-based, meaning if the GDT is 17 bytes in size, the limit will be 16 (0x10).
// Base: A 32-bit value that holds the address where the GDT starts.

#[repr(C, packed)]
struct GDT_Register {
    size: u16,
    offset: u32,
}

unsafe fn load_gdt() {
    let gdt_register = GDT_Register {
        size: (core::mem::size_of::<GDT>() - 1) as u16,
        offset: GDT.as_ptr() as u32,
    };

    asm!("lgdt [{}]", in(reg) &gdt_register, options(readonly, nostack, preserves_flags));
}

// https://wiki.osdev.org/Segmentation#Real_mode

unsafe fn load_kernel_code_segment() {
    asm!(
        "push 0x08", // Selector for code segment
        "push offset 1f", // Offset of next instruction
        "retf", // Jump to next instruction
        "1:", // Label for the next instruction
        // Update data segment registers
        "mov ax, 0x10", // Selector for data segment
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // Update stack segment register
        "mov ax, 0x18", // Selector for stack segment
        "mov ss, ax"
    );
}

pub fn gdt_init() {
	unsafe {
		load_gdt();
        load_kernel_code_segment();
    }
}
