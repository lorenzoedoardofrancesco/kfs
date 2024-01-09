use crate::boot::multiboot::{MultibootMemoryMapEntry, MultibootMemoryMapTag};
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_REGIONS: usize = 10;
const PMMNGR_BLOCK_SIZE: u32 = 4096; // 4KiB
const PMMNGR_BLOCKS_PER_BYTE: u32 = 8;
const USED_BLOCK: u32 = 0xffffffff;

pub static mut KERNEL_SPACE_START: u32 = 0;
pub static mut KERNEL_SPACE_END: u32 = 0;
pub static mut USER_SPACE_START: u32 = 0;
pub static mut USER_SPACE_END: u32 = 0;

#[derive(Clone, Copy)]
pub struct MemoryRegion {
	pub start_address: usize,
	pub size: usize,
}

/// Prevent the compiler from implementing Send and Sync on the PhysicalMemoryManager. For thread safety.
unsafe impl Send for PhysicalMemoryManager {}
unsafe impl Sync for PhysicalMemoryManager {}
/// Physical Memory Manager
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
		self.max_blocks = self.memory_size / PMMNGR_BLOCK_SIZE;
		self.memory_map_size = self.max_blocks / PMMNGR_BLOCKS_PER_BYTE;

		self.memory_map = unsafe {
			core::slice::from_raw_parts_mut(
				&_kernel_end as *const u8 as *mut u32,
				self.memory_map_size as usize,
			)
		};

		for i in 0..self.memory_map_size as usize {
			self.memory_map[i] = USED_BLOCK;
		}
		self.used_blocks = self.max_blocks;

		for i in 1..self.usable_regions.len() {
			// verifier si la premiere region est bien utilisable miao
			let region = self.usable_regions[i];
			if region.size == 0 {
				break;
			}
			self.init_region(region.start_address as u32, region.size as u32);
		}
	}

	/// Sets a bit in the memory map.
	fn mmap_set(&mut self, bit: u32) {
		let index = bit / 32;
		let offset = bit % 32;
		self.memory_map[index as usize] |= 1 << offset;
		self.used_blocks += 1;
	}

	/// Unsets a bit in the memory map.
	fn mmap_unset(&mut self, bit: u32) {
		let index = bit / 32;
		let offset = bit % 32;
		self.memory_map[index as usize] &= !(1 << offset);
		self.used_blocks -= 1;
	}

	/// Tests if a bit is set.
	fn mmap_test(&mut self, bit: u32) -> bool {
		let index = bit / 32;
		let offset = bit % 32;
		(self.memory_map[index as usize] & (1 << offset)) != 0
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

	/// Initializes a region of memory for use. Needs address u32 and size in bytes.
	fn init_region(&mut self, region_address: u32, region_size: u32) {
		let start_block = region_address / PMMNGR_BLOCK_SIZE;
		let mut blocks = region_size / PMMNGR_BLOCK_SIZE;

		if region_size % PMMNGR_BLOCK_SIZE != 0 {
			blocks += 1;
		}

		for block in start_block..start_block + blocks {
			self.mmap_unset(block);
		}
	}

	fn unset_region(&mut self, region_address: u32, region_size: u32) {
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
		if self.used_blocks >= self.max_blocks {
			return Err("Out of memory");
		}

		let frame = self.mmap_first_free();
		println!("Frame: {:#x}", frame);
		if frame != 0 {
			self.mmap_set(frame);
			Ok(frame * PMMNGR_BLOCK_SIZE)
		} else {
			Err("Out of memory")
		}
	}

	pub fn free_frame(&mut self, address: Result<u32, &'static str>) {
		// mettre un Result parce que si l'adresse est pas utilisable on fait quoi ?
		let address = address.unwrap();
		if self.is_address_usable(address) {
			self.mmap_unset(address / PMMNGR_BLOCK_SIZE);
		}
	}

	pub fn allocate_multiple_frames(&mut self, pages: u32) -> Result<u32, &'static str> {
		if self.used_blocks + pages >= self.max_blocks {
			return Err("Out of memory");
		}

		let mut frame = 0;
		let mut count = 0;
		for i in 0..self.max_blocks {
			if !self.mmap_test(i) {
				count += 1;
				if count == pages {
					frame = i - pages + 1;
					break;
				}
			} else {
				count = 0;
			}
		}

		if frame != 0 {
			for i in frame..frame + pages {
				self.mmap_set(i);
			}
			Ok(frame * PMMNGR_BLOCK_SIZE)
		} else {
			Err("Out of memory")
		}
	}

	fn init_available_memory(&mut self, mmap: &MultibootMemoryMapTag) {
		for i in 0..(mmap.size - mmap.entry_size) / mmap.entry_size {
			let entry: &MultibootMemoryMapEntry =
				unsafe { &*mmap.entries.as_ptr().add(i as usize) };
			if entry.entry_type == 1 {
				self.init_region(entry.address as u32, entry.len as u32);
			}
		}
	}

	fn print_values(&self) {
		println_serial!(
			"Physical memory manager: {} blocks available",
			self.max_blocks
		);
		println_serial!("Physical memory manager: {} blocks used", self.used_blocks);
		println_serial!(
			"Physical memory manager: {:p} memory map address",
			self.memory_map
		);
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

		unsafe {
			KERNEL_SPACE_START = &_kernel_start as *const u8 as u32;
			KERNEL_SPACE_END = &_kernel_end as *const u8 as u32 + 0x100000;
			USER_SPACE_START = KERNEL_SPACE_END;
			USER_SPACE_END = self.usable_regions[1].start_address as u32 + self.usable_regions[1].size as u32;

			println_serial!("Kernel space start: {:#x}", KERNEL_SPACE_START);
			println_serial!("Kernel space end: {:#x}", KERNEL_SPACE_END);
			println_serial!("User space start: {:#x}", USER_SPACE_START);
			println_serial!("User space end: {:#x}", USER_SPACE_END);
		}


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
					bits[31 - j] = '1';
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
}

pub fn physical_memory_manager_init() {
	let mut pmm = PMM.lock();

	pmm.process_memory_map();
	pmm.init();
	//pmm.print_memory_map();
}
