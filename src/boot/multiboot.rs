const MULTIBOOT_HEADER_MAGIC: u32 = 0xe85250d6;
const MULTIBOOT_HEADER_ARCHITECTURE: u32 = 0;
const MULTIBOOT_HEADER_CHECKSUM: u32 = (0_u32)
	.wrapping_sub(MULTIBOOT_HEADER_MAGIC)
	.wrapping_sub(MULTIBOOT_HEADER_ARCHITECTURE);
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x36d76289;

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

pub fn validate_multiboot(magic: u32, address: u32) {
	if magic != MULTIBOOT_BOOTLOADER_MAGIC {
		panic!("Invalid multiboot magic number: {:#x}", magic);
	}

	if address & 0x7 != 0 {
		panic!("Unaligned multiboot address: {:#x}", address);
	}
}