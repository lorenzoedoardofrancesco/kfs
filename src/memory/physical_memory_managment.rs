use core::mem::size_of;

use super::kmalloc::kernel_heap_init;
use super::kmalloc::{KERNEL_HEAP_END, KERNEL_HEAP_START};
use super::page_directory::init_page_directory;
use super::page_directory::{PAGE_DIRECTORY_ADDR, PAGE_TABLES_ADDR, PAGE_TABLE_SIZE};
use crate::boot::multiboot::{MultibootMemoryMapEntry, MultibootMemoryMapTag};
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_REGIONS: usize = 10;
const PMMNGR_BLOCK_SIZE: u32 = 4096; // 4KiB
const PMMNGR_BLOCKS_PER_INDEX: u32 = 32;
const USED_BLOCK: u32 = 0xFFFFFFFF;

pub const HIGH_KERNEL_OFFSET: u32 = 0xC0000000;
pub const KERNEL_HEAP_SIZE: u32 = 0x100000; // todo meow meow meow

const USER_SPACE_START: u32 = 0;
const USER_SPACE_END: u32 = HIGH_KERNEL_OFFSET - 1;
const KERNEL_SPACE_START: u32 = HIGH_KERNEL_OFFSET;
const KERNEL_SPACE_END: u32 = 0xFFFFFFFF;

pub static mut PMM_ADDRESS: u32 = 0;

pub static mut MEMORY_MAP: u32 = 0;
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
	pub start_address: usize,
	pub size: usize,
}

/// Prevent the compiler from implementing Send and Sync on the PhysicalMemoryManager. For thread safety.
unsafe impl Send for PhysicalMemoryManager {}
unsafe impl Sync for PhysicalMemoryManager {}
/// Physical Memory Manager
#[derive(Debug)]
pub struct PhysicalMemoryManager {
	memory_map: &'static mut [u32],
	used_blocks: u32,
	max_blocks: u32,
	memory_map_size: u32,
	pub usable_regions: [MemoryRegion; MAX_REGIONS],
	pub memory_size: u32,
	pub memory_map_tag: Option<&'static MultibootMemoryMapTag>,
	pub memory_map_entries: Option<&'static [MultibootMemoryMapEntry]>,
}

lazy_static! {
	pub static ref PMM: Mutex<PhysicalMemoryManager> = Mutex::new(PhysicalMemoryManager {
		memory_map: unsafe { core::slice::from_raw_parts_mut(0 as *mut u32, 0) },
		used_blocks: 0,
		max_blocks: 0,
		memory_map_size: 0,
		usable_regions: [MemoryRegion {
			start_address: 0,
			size: 0,
		}; 10],
		memory_size: 0,
		memory_map_tag: None,
		memory_map_entries: None,
	});
}

extern "C" {
	static mut _kernel_start: u8;
	static mut _kernel_end: u8;
}

impl PhysicalMemoryManager {
	pub fn init(&mut self) {
		let max_blocks = self.memory_size / PMMNGR_BLOCK_SIZE;
		let memory_map_size = max_blocks / PMMNGR_BLOCKS_PER_INDEX;

		println_serial!("Initializing Physical Memory Manager");
		println_serial!("PMM struct {:?}", self);
		println_serial!("PMM memory size: {:#x}", self.memory_size);
		unsafe {
			MEMORY_MAP = &_kernel_end as *const u8 as u32;
			PMM_ADDRESS = align_up(MEMORY_MAP + memory_map_size);
			PAGE_DIRECTORY_ADDR = align_up(PMM_ADDRESS + size_of::<PhysicalMemoryManager>() as u32);
			PAGE_TABLES_ADDR = PAGE_DIRECTORY_ADDR + 0x1000;
			KERNEL_HEAP_START = (PAGE_TABLES_ADDR + PAGE_TABLE_SIZE as u32 + 0x1000) as *mut u8;
			KERNEL_HEAP_END = KERNEL_HEAP_START.wrapping_add(KERNEL_HEAP_SIZE as usize);

			println_serial!("User space start:         {:#x}", USER_SPACE_START);
			println_serial!("User space end:           {:#x}", USER_SPACE_END);
			println_serial!("Kernel space start:       {:#x}", KERNEL_SPACE_START);
			println_serial!("Memory map address:       {:#x}", MEMORY_MAP);
			println_serial!("PMM address:              {:#x}", PMM_ADDRESS);
			println_serial!("Page directory address:   {:#x}", PAGE_DIRECTORY_ADDR);
			println_serial!("Page tables address:      {:#x}", PAGE_TABLES_ADDR);
			println_serial!("Kernel heap start:        {:#x}", KERNEL_HEAP_START as u32);
			println_serial!("Kernel heap end:          {:#x}", KERNEL_HEAP_END as u32);
			println_serial!("Kernel space end:         {:#x}", KERNEL_SPACE_END);
		}

		let pmm: &mut PhysicalMemoryManager =
			unsafe { &mut *(PMM_ADDRESS as *mut PhysicalMemoryManager) };
		*pmm = PhysicalMemoryManager {
			memory_map: unsafe {
				core::slice::from_raw_parts_mut(MEMORY_MAP as *mut u32, memory_map_size as usize)
			},
			used_blocks: 0,
			max_blocks: self.memory_size / PMMNGR_BLOCK_SIZE,
			memory_map_size: memory_map_size,
			usable_regions: self.usable_regions,
			memory_size: self.memory_size,
			memory_map_tag: None,
			memory_map_entries: None,
		};

		println_serial!(
			"Memory size: {:#x}, max blocks: {:#x}, memory map size: {:#x}",
			pmm.memory_size,
			pmm.max_blocks,
			pmm.memory_map_size
		);

		pmm.memory_map = unsafe {
			core::slice::from_raw_parts_mut(
				&_kernel_end as *const u8 as *mut u32,
				pmm.memory_map_size as usize,
			)
		};

		for i in 0..self.memory_map_size as usize {
			pmm.memory_map[i] = USED_BLOCK;
		}
		pmm.used_blocks = pmm.max_blocks;

		for i in 1..pmm.usable_regions.len() {
			let region = pmm.usable_regions[i];
			if region.size == 0 {
				break;
			}
			pmm.set_region_as_available(region.start_address as u32, region.size as u32);
		}
		// Set the kernel space as used
		pmm.set_region_as_unavailable(KERNEL_SPACE_START - HIGH_KERNEL_OFFSET, unsafe {
			KERNEL_HEAP_START as u32 - KERNEL_SPACE_START - 1
		});
	}

	/// Sets a bit in the memory map.
	fn mmap_set(&mut self, bit: u32) {
		let pmm: &mut PhysicalMemoryManager =
			unsafe { &mut *(PMM_ADDRESS as *mut PhysicalMemoryManager) };

		let index = bit / 32;
		let offset = bit % 32;
		pmm.memory_map[index as usize] |= 1 << offset;
		pmm.used_blocks += 1;
	}

	/// Unsets a bit in the memory map.
	fn mmap_unset(&mut self, bit: u32) {
		let index = bit / 32;
		let offset = bit % 32;
		let pmm: &mut PhysicalMemoryManager =
			unsafe { &mut *(PMM_ADDRESS as *mut PhysicalMemoryManager) };
		pmm.memory_map[index as usize] &= !(1 << offset);
		pmm.used_blocks -= 1;
	}

	fn mmap_unset_address(&mut self, address: u32) {
		let bit = address / PMMNGR_BLOCK_SIZE;
		self.mmap_unset(bit);
	}

	/// Tests if a bit is set.
	fn mmap_test(&mut self, bit: u32) -> bool {
		let index = bit / 32;
		let offset = bit % 32;
		(self.memory_map[index as usize] & (1 << offset)) != 0
	}

	pub fn nmap_test_address(&mut self, address: u32) -> bool {
		let bit = address / PMMNGR_BLOCK_SIZE;
		self.mmap_test(bit)
	}

	fn mmap_first_free(&mut self) -> u32 {
		for i in 0..self.max_blocks / 32 {
			if self.memory_map[i as usize] != 0xffffffff {
				for j in 0..32 {
					let bit: u32 = 1 << j;
					if (self.memory_map[i as usize] & bit) == 0 {
						return i * 32 + j;
					}
				}
			}
		}
		0
	}

	/// Sets a region of memory as available. This is used to mark the usable regions of memory
	fn set_region_as_available(&mut self, region_address: u32, region_size: u32) {
		let start_block = region_address / PMMNGR_BLOCK_SIZE;
		let mut blocks = region_size / PMMNGR_BLOCK_SIZE;

		if region_size % PMMNGR_BLOCK_SIZE != 0 {
			blocks += 1;
		}

		for block in start_block..start_block + blocks {
			self.mmap_unset(block);
		}
	}

	/// Sets a region of memory as used. Needs address u32 and size in bytes.
	fn set_region_as_unavailable(&mut self, region_address: u32, region_size: u32) {
		let start_block = region_address / PMMNGR_BLOCK_SIZE;
		let mut blocks = region_size / PMMNGR_BLOCK_SIZE;

		if region_size % PMMNGR_BLOCK_SIZE != 0 {
			blocks += 1;
		}

		for block in start_block..start_block + blocks {
			self.mmap_set(block);
			self.used_blocks += 1;
		}
	}

	pub fn allocate_frame(&mut self) -> Result<u32, &'static str> {
		println_serial!(
			"Used blocks: {:#x}, Max blocks: {:#x}",
			self.used_blocks,
			self.max_blocks
		);
		let pmm = unsafe { &mut *(PMM_ADDRESS as *mut PhysicalMemoryManager) };

		if pmm.used_blocks >= pmm.max_blocks {
			return Err("Out of memory");
		}

		let mut frame = 0;
		'outer: for i in 0..pmm.max_blocks / 32 {
			if pmm.memory_map[i as usize] != 0xffffffff {
				for j in 0..32 {
					let bit: u32 = 1 << j;
					if (pmm.memory_map[i as usize] & bit) == 0 {
						frame = i * 32 + j;
						break 'outer;
					}
				}
			}
		}
		println_serial!("Frame: {:#x}", frame);
		if frame != 0 {
			pmm.mmap_set(frame);
			Ok(frame * PMMNGR_BLOCK_SIZE)
		} else {
			Err("Out of memory")
		}
	}

	pub fn deallocate_frame(&mut self, address: u32) {
		if self.is_address_usable(address) {
			self.mmap_unset(address / PMMNGR_BLOCK_SIZE);
		}
	}

	fn process_memory_map(&mut self) {
		let memory_map_entries: &[MultibootMemoryMapEntry] = self.memory_map_entries.unwrap();

		let mut i = 0;
		println_serial!("      Memory map entry: ");
		for entry in memory_map_entries {
			println_serial!(
				"      Address: 0x{:08x} | Length: 0x{:07x} | Type: {:#x} ({})",
				entry.address,
				entry.len,
				entry.entry_type,
				match entry.entry_type {
					1 => "Usable",
					2 => "Reserved",
					3 => "ACPI Reclaimable",
					4 => "ACPI NVS",
					5 => "Bad memory",
					_ => "Unknown",
				}
			);
			if entry.entry_type == 1 {
				self.usable_regions[i] = MemoryRegion {
					start_address: entry.address as usize,
					size: entry.len as usize,
				};
				i += 1;
			}
		}

		self.memory_size = memory_map_entries.last().unwrap().address as u32
			+ memory_map_entries.last().unwrap().len as u32;
	}

	fn is_address_usable(&self, address: u32) -> bool {
		for region in self.usable_regions.iter() {
			if address >= region.start_address as u32
				&& address <= region.start_address as u32 + region.size as u32
			{
				return true;
			}
		}
		false
	}

	pub fn print_memory_map(&self) {
		println_serial!("Memory Map:");
		for index in 0..(self.memory_map_size as usize) {
			let block = self.memory_map[index]; // Access the block directly using index

			let mut bits: [char; 32] = ['0'; 32];

			for j in 0..32 {
				if block & (1 << j) != 0 {
					bits[j] = '1';
				}
			}

			// Printing each block's address and its bit pattern directly
			print_serial!("0x{:08x}: ", index * 32 * PMMNGR_BLOCK_SIZE as usize);
			for bit in bits.iter() {
				print_serial!("{}", bit);
			}
			println_serial!();
		}
	}

	pub fn update_bitmap_from_memory(&mut self) {
		// Iterate over the entire memory range in block-size increments
		for address in (0..self.memory_size).step_by(PMMNGR_BLOCK_SIZE as usize) {
			// Check if the memory block (frame) at this address is used
			if self.is_block_used(address) {
				// Calculate the corresponding bit in the bitmap
				let bit = address / PMMNGR_BLOCK_SIZE;
				// Set the bit to mark the block as used
				self.mmap_set(bit);
			}
		}
	}

	fn is_block_used(&self, address: u32) -> bool {
		let block_ptr = address as *const u8; // Pointer to the start of the block

		for offset in 0..(PMMNGR_BLOCK_SIZE as isize) {
			// Check each byte in the block
			unsafe {
				if block_ptr.offset(offset).read_volatile() != 0 {
					// If any byte is non-zero, the block is used
					return true;
				}
			}
		}
		false
	}
}

pub fn physical_memory_manager_init() {
	let mut pmm = PMM.lock();

	pmm.process_memory_map();
	pmm.init();
	unsafe {
		//	kernel_heap_init();
	}
	//pmm.print_memory_map();
}

/// Align an address to the nearest page boundary.
pub fn align_up(addr: u32) -> u32 {
	(addr + 0xfff) & !0xfff
}
