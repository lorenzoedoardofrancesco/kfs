//! # Global Descriptor Table Module (GDT)
//!
//! This module defines the structures and functions for setting up and managing the
//! GDT in a low-level context for an operating system kernel.
//! The GDT is a crucial part of x86 and x86_64 architectures
//! for defining different memory segments and access rights.
//!
//! ## Overview
//!
//! The GDT is used by the CPU to determine the correct memory segment to use
//! for a given memory access. The GDT is loaded into the CPU's GDTR register
//! using the `lgdt` instruction. It includes definitionsfor standard segments
//! like kernel code and data segments, user segments, and stack segments.
use crate::structures::accessflags::{
	KERNEL_CODE_SEGMENT, KERNEL_DATA_SEGMENT, KERNEL_STACK_SEGMENT, MAX_SEGMENT_SIZE, NO_OFFSET,
	NULL_SEGMENT, SEGMENT_FLAGS, USER_CODE_SEGMENT, USER_DATA_SEGMENT, USER_STACK_SEGMENT,
};
use crate::utils::debug::LogLevel;
use core::arch::asm;

/// Represents an entry in the Global Descriptor Table (GDT).
///
/// Each GDT entry is used to define the characteristics of a memory segment,
/// including its size, base address, and access rights.
#[repr(C, packed)]
pub struct GdtEntry {
	limit_low: u16,
	base_low: u16,
	base_middle: u8,
	access: u8,
	flags: u8,
	base_high: u8,
}

/// Creates a new GDT entry.
///
/// # Arguments
///
/// * `limit` - The size limit of the segment.
/// * `base` - The base address of the segment.
/// * `access` - Access rights flags.
/// * `flags` - Additional flags (such as granularity).
/// * `name` - Descriptive name for the segment (used for debugging).
///
macro_rules! create_gdt_entry {
	($limit:expr, $base:expr, $access:expr, $flags:expr, $name:expr) => {
		GdtEntry {
			limit_low: ($limit & 0xffff) as u16,
			base_low: ($base & 0xffff) as u16,
			base_middle: (($base >> 16) & 0xff) as u8,
			access: $access,
			flags: ($flags & 0xf0) | ((($limit >> 16) & 0x0f) as u8),
			base_high: (($base >> 24) & 0xff) as u8,
		}
	};
}

/// Static initialization of the Global Descriptor Table (GDT).
///
/// This block sets up the GDT with predefined segments for kernel and user
/// modes, including code, data, and stack segments.
#[link_section = ".gdt"]
static LOW_GDT: [GdtEntry; 7] = [
	create_gdt_entry!(0, 0, NULL_SEGMENT, 0, "NULL segment"),
	create_gdt_entry!(
		MAX_SEGMENT_SIZE,
		NO_OFFSET,
		KERNEL_CODE_SEGMENT,
		SEGMENT_FLAGS,
		"Kernel code segment"
	),
	create_gdt_entry!(
		MAX_SEGMENT_SIZE,
		NO_OFFSET,
		KERNEL_DATA_SEGMENT,
		SEGMENT_FLAGS,
		"Kernel data segment"
	),
	create_gdt_entry!(
		MAX_SEGMENT_SIZE,
		NO_OFFSET,
		KERNEL_STACK_SEGMENT,
		SEGMENT_FLAGS,
		"Kernel stack segment"
	),
	create_gdt_entry!(
		MAX_SEGMENT_SIZE,
		NO_OFFSET,
		USER_CODE_SEGMENT,
		SEGMENT_FLAGS,
		"User code segment"
	),
	create_gdt_entry!(
		MAX_SEGMENT_SIZE,
		NO_OFFSET,
		USER_DATA_SEGMENT,
		SEGMENT_FLAGS,
		"User data segment"
	),
	create_gdt_entry!(
		MAX_SEGMENT_SIZE,
		NO_OFFSET,
		USER_STACK_SEGMENT,
		SEGMENT_FLAGS,
		"User stack segment"
	),
];

pub static mut GDT: *mut [GdtEntry; 7] = core::ptr::null_mut();
/// Represents the register structure used for loading the GDT.
///
/// This structure is required for the `lgdt` instruction which loads
/// the address and size of the GDT into the CPU's GDTR register.
#[repr(C, packed)]
pub struct GdtRegister {
	size: u16, 
	offset: u32,
}

/// Loads the GDT.
fn load_gdt() {
	unsafe {
		let gdt_register = GdtRegister {
			size: (core::mem::size_of::<[GdtEntry; 7]>() - 1) as u16,
			offset: GDT as u32,
		};

		asm!("lgdt [{}]", in(reg) &gdt_register, options(readonly, nostack, preserves_flags));
	}
}

/// Loads the data segments.
///
/// This function updates the data segment registers (ds, es, fs, gs)
/// of the CPU to use the segments defined in the GDT.
///
/// # Safety
///
/// This function is unsafe because it directly manipulates CPU segment registers.
fn load_data_segments() {
	unsafe {
		asm!(
			"mov ax, 0x10", // Kernel data segments
			"mov ds, ax",
			"mov es, ax",
			"mov fs, ax",
			"mov gs, ax",
			options(nostack, preserves_flags)
		);
	}
}

/// Loads the stack segment.
///
/// This function updates the stack segment register (ss) of the CPU
/// to use the segment defined in the GDT. This is crucial for ensuring
/// that the CPU uses the correct stack for operations after this point.
///
/// # Safety
///
/// This function is unsafe because it directly manipulates the CPU's
/// stack segment register, which is critical for proper CPU operations.
fn load_stack_segment() {
	unsafe {
		asm!(
			"mov ax, 0x18", // Kernel stack segment
			"mov ss, ax",
			options(nostack, preserves_flags)
		);
	}
}

/// Loads the code segment.
///
/// This function updates the code segment register (cs) of the CPU.
/// It requires a far jump to ensure that the CPU starts fetching
/// and decoding instructions from the new code segment as defined
/// in the GDT. This is essential for the CPU to execute subsequent
/// instructions correctly.
///
/// # Safety
///
/// This function is unsafe because it involves a far jump which
/// changes the code segment register (cs). Incorrectly setting the
/// code segment can lead to undefined and erroneous CPU behavior.
fn load_code_segment() {
	unsafe {
		asm!(
			"push 0x08", // Kernel code segment
			"lea eax, [1f]",
			"push eax",
			"retf",
			"1:",
			options(nostack, preserves_flags)
		);
	}
}

/// Initializes the Global Descriptor Table (GDT).
///
/// This function sets up the GDT with necessary segments and
/// loads it into the CPU. It also updates the segment registers
/// to use the new segments.
pub fn init() {
	unsafe {
		GDT = (&LOW_GDT as *const _ as usize + 0xC0000000) as *mut [GdtEntry; 7];
	}
	load_gdt();
	log!(
		LogLevel::Info,
		"GDT successfully loaded at 0x{:08x}",
		unsafe { GDT as *const _ as usize }
	);
	load_data_segments();
	load_stack_segment();
	load_code_segment();
	log!(
		LogLevel::Info,
		"Kernel data, stack and code segments successfully loaded"
	);
}
