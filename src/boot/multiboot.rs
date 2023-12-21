//! Module for handling the Multiboot header.
//!
//! The Multiboot header is used in bootloading an operating system kernel
//! that is compliant with the Multiboot specification. This module defines
//! the structure of the Multiboot header and provides functionality for
//! validating the Multiboot boot information provided by the bootloader.
//!
//! ## Overview
//!
//! The Multiboot header is a structure that is placed at the beginning of
//! the kernel image by the bootloader. It contains information about the
//! system memory map, the video mode, and the initial ramdisk. The header
//! is used by the kernel to determine the available memory and to locate
//! the initial ramdisk. The header is also used by the bootloader to
//! determine the entry point of the kernel.

const MULTIBOOT_HEADER_MAGIC: u32 = 0xe85250d6;
const MULTIBOOT_HEADER_ARCHITECTURE: u32 = 0;
const MULTIBOOT_HEADER_CHECKSUM: u32 = (0_u32)
	.wrapping_sub(MULTIBOOT_HEADER_MAGIC)
	.wrapping_sub(MULTIBOOT_HEADER_ARCHITECTURE);
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x36d76289;

/// Static Multiboot header.
#[used]
#[link_section = ".multiboot_header"]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
	magic: MULTIBOOT_HEADER_MAGIC,
	architecture: MULTIBOOT_HEADER_ARCHITECTURE,
	header_length: core::mem::size_of::<MultibootHeader>() as u32,
	checksum: MULTIBOOT_HEADER_CHECKSUM
		.wrapping_sub(core::mem::size_of::<MultibootHeader>() as u32),
	end_tag_type: 0,
	end_tag_flags: 0,
	end_tag_size: 8,
};

/// Structure representing the Multiboot header.
///
/// The Multiboot header consists of several fields including a magic number,
/// architecture type, length of the header, checksum, and end tag details.
/// It must be placed at the beginning of the text segment of the kernel binary.
#[repr(C)]
pub struct MultibootHeader {
	magic: u32,
	architecture: u32,
	header_length: u32,
	checksum: u32,
	end_tag_type: u16,
	end_tag_flags: u16,
	end_tag_size: u32,
}

/// Validates the Multiboot information.
///
/// This function checks the magic number provided by the bootloader against
/// the expected Multiboot magic number and validates the address alignment.
pub fn validate_multiboot(magic: u32, address: u32) {
	if magic != MULTIBOOT_BOOTLOADER_MAGIC {
		panic!("Invalid multiboot magic number: {:#x}", magic);
	}

	if address & 0x7 != 0 {
		panic!("Unaligned multiboot address: {:#x}", address);
	}
}
