use lazy_static::lazy_static;
use spin::Mutex;
use crate::boot::multiboot::{MultibootMemoryMapEntry, MultibootMemoryMapTag, MultibootTagBasicMemInfo};

const PMMNGR_BLOCK_SIZE: u32 = 4096; // 4KB
const PMMNGR_BLOCKS_PER_BYTE: u32 = 8;


lazy_static! {
    pub static ref PMM: Mutex<PhysicalMemoryManager> = Mutex::new(PhysicalMemoryManager::new());
}

/// Prevent the compiler from implementing Send and Sync on the PhysicalMemoryManager. For thread safety.
unsafe impl Send for PhysicalMemoryManager {}
unsafe impl Sync for PhysicalMemoryManager {}

// Location of the end of the kernel, we will put the memory map here.  See linker.ld
extern "C" {
    static _kernel_end: u8;
}


/// Physical Memory Manager
pub struct PhysicalMemoryManager {
	memory_map: &'static mut [u32],
	used_blocks: u32,
	max_blocks: u32,
	size: u32,
}

impl PhysicalMemoryManager {
	fn new() -> Self {
		PhysicalMemoryManager {
			memory_map: &mut [],
			used_blocks: 0,
			max_blocks: 0,
			size: 0,
		}
	}

	fn init(
		&mut self,
		boot_info: &MultibootTagBasicMemInfo,
		memory_map_tag: &MultibootMemoryMapTag,
	) {
		let actual_memory = (boot_info.mem_upper - boot_info.mem_lower) * 1024;
		self.max_blocks = actual_memory / PMMNGR_BLOCK_SIZE;
		self.used_blocks = self.max_blocks;
		// We get the address of the end of the kernel.
		self.memory_map =
			unsafe { core::slice::from_raw_parts_mut(&_kernel_end as *const u8 as *mut u32, (self.max_blocks / PMMNGR_BLOCKS_PER_BYTE) as usize) };
		self.size = self.max_blocks / PMMNGR_BLOCKS_PER_BYTE;
		// Mark all blocks as used. Kernel will mark the ones that are free later on to prevent memory corruption.
		for i in 0..self.size {
			self.memory_map[i as usize] = 0xffffffff;
		}

		// println_serial!("Physical memory manager: {} blocks available", self.max_blocks);
		// println_serial!("Physical memory manager: {} blocks used", self.used_blocks);
		// println_serial!("Physical memory manager: {:#?} memory map address", self.memory_map);

		self.init_available_memory(memory_map_tag);
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
        let mut align = region_address / PMMNGR_BLOCK_SIZE;
        let blocks = region_size / PMMNGR_BLOCK_SIZE;
        // if region_size % PMMNGR_BLOCK_SIZE != 0 {
        //     blocks += 1;
        // }
        for _ in 0..blocks {
            self.mmap_unset(align);
            align += 1;
        }
        self.used_blocks -= blocks as u32;

    }

	fn unset_region() {//TODO
	}

    fn alloc_block(&mut self) -> u32 {
        if self.used_blocks >= self.max_blocks {
            panic!("Out of memory");
        }

        let frame = self.mmap_first_free();
        if frame != 0 {
            self.mmap_set(frame);
        } else {
            panic!("Out of memory");
        }

        frame * PMMNGR_BLOCK_SIZE
    }

    fn free_block(&mut self, block: u32) {
        self.mmap_unset(block / PMMNGR_BLOCK_SIZE);
    }

	fn init_available_memory(&mut self, mmap: &MultibootMemoryMapTag) {
		for i in 0..(mmap.size - mmap.entry_size) / mmap.entry_size {
			let entry: &MultibootMemoryMapEntry = unsafe { &*mmap.entries.as_ptr().add(i as usize) };
			if entry.entry_type == 1 {
				self.init_region(entry.addr as u32, entry.len as u32);
			}
		}
	}

	fn print_values(&self) {
		println_serial!("Physical memory manager: {} blocks available", self.max_blocks);
		println_serial!("Physical memory manager: {} blocks used", self.used_blocks);
		println_serial!("Physical memory manager: {:p} memory map address", self.memory_map);
	}
}

/// Je veux rajouter le MultibootMemoryMapEntry mais je ne sais pas comment faire encore.
pub fn physical_memory_init(boot_info: &MultibootTagBasicMemInfo, memory_map_tag: &MultibootMemoryMapTag) {

    PMM.lock().init(boot_info, memory_map_tag);
	PMM.lock().print_values(); 
}
