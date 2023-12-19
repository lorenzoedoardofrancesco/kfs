const PMMNGR_BLOCK_SIZE: u32 = 4096; // 4KB
const PMMNGR_BLOCKS_PER_BYTE: u32 = 8;

/// Physical Memory Manager
struct PhysicalMemoryManager {
	memory_map: &'static mut [u32],
	used_blocks: u32,
	max_blocks: u32,
}

impl PhysicalMemoryManager {
	fn new() -> Self {
		PhysicalMemoryManager {
			memory_map: &mut [],
			used_blocks: 0,
			max_blocks: 0,
		}
	}

	fn init(
		&'static mut self,
		actual_memory_kb: u32,
		bitmap_start_address: *mut u32,
		bitmap_size: u32,
	) {
		let actual_memory = actual_memory_kb * 1024;
		self.max_blocks = actual_memory / PMMNGR_BLOCK_SIZE;
		self.used_blocks = self.max_blocks;
		self.memory_map =
			unsafe { core::slice::from_raw_parts_mut(bitmap_start_address, bitmap_size) };

		/// Mark all blocks as used. Kernel will mark the ones that are free later on to prevent memory corruption.
		for i in 0..bitmap_size {
			self.memory_map[i] = 0xffffffff;
		}
	}

	/// Sets a bit in the memory map.
	fn mmap_set(&mut self, bit: u32) {
		let idx = bit / 32;
		let offset = bit % 32;
		self.memory_map[idx] |= 1 << offset;
        self.used_blocks += 1;
	}

	/// Unsets a bit in the memory map.
	fn mmap_unset(&mut self, bit: u32) {
		let idx = bit / 32;
		let offset = bit % 32;
		self.memory_map[idx] &= !(1 << offset);
        self.used_blocks -= 1;
	}

	/// Tests if a bit is set.
	fn mmap_test(&self, bit: u32) -> bool {
		let idx = bit / 32;
		let offset = bit % 32;
		(self.memory_map[idx] & (1 << offset)) != 0
	}

    fn mmap_first_free(&self) -> Option<u32> {
        for i in 0..self.max_blocks / 32 {
            if self.memory_map[i] != 0xffffffff {
                for j in 0..32 {
                    let bit: u32 = 1 << j;
                    if (self.memory_map[i] & bit) == 0 {
                        return Some(i * 32 + j);
                    }
                }
            }
        }
        None
    }

    /// Initializes a region of memory for use. Needs address and size.
    fn init_region(&self, region_address: u32, region_size: u32) {
        let mut align = region_address / PMMNGR_BLOCK_SIZE;
        let mut blocks = region_size / PMMNGR_BLOCK_SIZE;
        // if region_size % PMMNGR_BLOCK_SIZE != 0 {
        //     blocks += 1;
        // }
        for _ in 0..blocks {
            self.mmap_unset(align);
            align += 1;
        }
        self.used_blocks -= blocks as u32;

    }

    fn alloc_block(&self) -> u32 {
        if self.used_blocks >= self.max_blocks {
            panic!("Out of memory");
        }

        let frame = self.mmap_first_free();
        if let Some(frame) = frame {
            self.mmap_set(frame);
        } else {
            panic!("Out of memory");
        }

        frame * PMMNGR_BLOCK_SIZE
    }

    fn free_block(&self, block: u32) {
        self.mmap_unset(block / PMMNGR_BLOCK_SIZE);
    }
}

fn physical_memory_init(actual_memory_kb: u32, bitmap_address: u32, bitmap_size: u32) {
    
}
