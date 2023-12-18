use crate::memory::memory_management::{PAGE_TABLE_END, PAGE_TABLE_START};

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
struct MultibootHeader {
	magic: u32,
	architecture: u32,
	header_length: u32,
	checksum: u32,
	end_tag_type: u16,
	end_tag_flags: u16,
	end_tag_size: u32,
}

#[repr(C)]
struct MultibootInfo {
	total_size: u32,
	reserved: u32,
	tags: [MultibootTag; 1],
}

#[repr(C)]
struct MultibootTag {
	tag_type: u32,
	size: u32,
}

#[repr(C)]
struct MultibootTagString {
	tag_type: u32,
	size: u32,
	string: u8,
}

#[repr(C)]
struct MultibootTagBasicMemInfo {
	tag_type: u32,
	size: u32,
	mem_lower: u32,
	mem_upper: u32,
}

#[repr(C)]
struct MultibootTagBootDev {
	tag_type: u32,
	size: u32,
	biosdev: u32,
	partition: u32,
	sub_partition: u32,
}

#[repr(C)]
struct MultibootMemoryMapTag {
	tag_type: u32,
	size: u32,
	entry_size: u32,
	entry_version: u32,
	entries: [MultibootMemoryMapEntry; 1],
}

#[repr(C)]
struct MultibootMemoryMapEntry {
	addr: u64,
	len: u64,
	entry_type: u32,
	zero: u32,
}

const MULTIBOOT_TAG_TYPE_END: u32 = 0;

const MULTIBOOT_TAG_TYPE_CMDLINE: u32 = 1;
const MULTIBOOT_TAG_TYPE_BOOT_LOADER_NAME: u32 = 2;
const MULTIBOOT_TAG_TYPE_BASIC_MEMINFO: u32 = 4;
const MULTIBOOT_TAG_TYPE_BOOTDEV: u32 = 5;
const MULTIBOOT_TAG_TYPE_MMAP: u32 = 6;

pub fn strlen(s: *const u8) -> usize {
	let mut len = 0;
	while unsafe { *s.add(len) } != 0 {
		len += 1;
	}
	len
}

pub fn u8_to_str(pointer: *const u8) -> &'static str {
	unsafe {
		let length = strlen(pointer);
		let slice = core::slice::from_raw_parts(pointer, length);
		core::str::from_utf8(slice).unwrap()
	}
}

pub fn init(magic: u32, addr: u32) {
	if magic != MULTIBOOT_BOOTLOADER_MAGIC {
		panic!("Invalid multiboot magic number: {:#x}", magic);
	}

	if addr & 0x7 != 0 {
		panic!("Unaligned multiboot address: {:#x}", addr);
	}

	let multiboot_info: &MultibootInfo = unsafe { &*(addr as *const MultibootInfo) };
	println_serial!("Announced mbi size: {:#x}", multiboot_info.total_size);

	let mut current_tag: *const MultibootTag = multiboot_info.tags.as_ptr();
	let mut tag: &MultibootTag = unsafe { &*current_tag };

	while tag.tag_type != MULTIBOOT_TAG_TYPE_END {
		//println_serial!("Tag {:#x} size: {:#x}", tag.tag_type, tag.size);
		match tag.tag_type {
			MULTIBOOT_TAG_TYPE_CMDLINE => {
				let cmdline = unsafe { &*(current_tag as *const MultibootTagString) };
				println_serial!("Command line: {}", u8_to_str(&cmdline.string));
			}
			MULTIBOOT_TAG_TYPE_BOOT_LOADER_NAME => {
				let bootloader_name = unsafe { &*(current_tag as *const MultibootTagString) };
				println_serial!("Bootloader name: {}", u8_to_str(&bootloader_name.string));
			}
			MULTIBOOT_TAG_TYPE_BASIC_MEMINFO => {
				let meminfo = unsafe { &*(current_tag as *const MultibootTagBasicMemInfo) };
				println_serial!(
					"Mem lower: {}KB, Mem upper: {}KB",
					meminfo.mem_lower,
					meminfo.mem_upper
				);
			}
			MULTIBOOT_TAG_TYPE_BOOTDEV => {
				let bootdev = unsafe { &*(current_tag as *const MultibootTagBootDev) };
				println_serial!(
					"Boot device: {:#x}, {}, {}",
					bootdev.biosdev,
					bootdev.partition,
					bootdev.sub_partition
				);
			}
			MULTIBOOT_TAG_TYPE_MMAP => {
				let nmap: *const MultibootMemoryMapEntry = unsafe {
					(*(current_tag as *const MultibootMemoryMapTag))
						.entries
						.as_ptr()
				};
				let mmap = unsafe { &*(current_tag as *const MultibootMemoryMapTag) };
				println_serial!("Memory map:");
				for i in 0..(mmap.size - mmap.entry_size) / mmap.entry_size {
					let entry = unsafe { &*nmap.add(i as usize) };
					println_serial!(
						"  {:#x}-{:#x} type: {:#x}",
						entry.addr,
						entry.addr + entry.len,
						entry.entry_type
					);
				}

				let entries_count = (mmap.size - mmap.entry_size) / mmap.entry_size;
				let memory_map_entries =
					unsafe { core::slice::from_raw_parts(nmap, entries_count as usize) };
				process_memory_map(memory_map_entries);
			}
			_ => {}
		}
		current_tag = (current_tag as usize + (tag.size as usize + 7) & !7) as *const MultibootTag;
		tag = unsafe { &*current_tag };
	}
}

fn process_memory_map(memory_map_entries: &[MultibootMemoryMapEntry]) {
	let mut largest_region = (0, 0);
	for entry in memory_map_entries {
		if entry.entry_type == 1 {
			if entry.len > largest_region.1 {
				largest_region = (entry.addr, entry.len);
			}
		}
	}

	unsafe {
		PAGE_TABLE_START = largest_region.0 as usize;
		PAGE_TABLE_END = PAGE_TABLE_START + largest_region.1 as usize;
	}

	println!("Largest region: {:#x}-{:#x}", largest_region.0, largest_region.0 + largest_region.1);
}
