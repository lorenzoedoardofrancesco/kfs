//! Implementation of kmalloc, kfree, ksbrk, and ksize for memory allocation.
//! kmalloc and kfree manage memory in a simple linear heap.
//! ksbrk adjusts the heap size, and ksize returns the size of an allocated block.

use crate::memory::{
	page_directory::{ENTRY_COUNT, PAGE_DIRECTORY, PAGE_SIZE},
	page_table_entry::PageTableFlags,
	physical_memory_managment::{KERNEL_HEAP_SIZE, PMM},
};
use bitflags::bitflags;
use core::ptr;
use core::sync::atomic::Ordering;

use super::page_directory::{self, PageDirectoryFlags};

pub static mut KERNEL_HEAP_START: *mut u8 = 0 as *mut u8;
pub static mut KERNEL_HEAP_END: *mut u8 = 0 as *mut u8;
pub static mut KERNEL_HEAP_BREAK: *mut u8 = ptr::null_mut();

const MIN_ALLOCATION_SIZE: u32 = 32;
const MAX_ALLOCATION_SIZE: u32 = PAGE_SIZE as u32 * 1024 - KMALLOC_HEADER_SIZE as u32;

bitflags! {
	#[repr(C)]
	struct KmallocHeaderFlags: u32 {
		const USED = 1 << 31;
		const SIZE = 0x7FFFFFFF;
	}
}

const KMALLOC_HEADER_SIZE: usize = core::mem::size_of::<KmallocHeader>();

#[repr(C, packed)]
struct KmallocHeader {
	value: u32,
}

impl KmallocHeader {
	fn new(used: bool, size: u32) -> Self {
		Self {
			value: (size | (used as u32)) as u32,
		}
	}

	fn set_used(&mut self, used: bool) {
		if used {
			self.value |= KmallocHeaderFlags::USED.bits();
		} else {
			self.value &= !KmallocHeaderFlags::USED.bits();
		}
	}

	fn used(&self) -> bool {
		self.value & KmallocHeaderFlags::USED.bits() != 0
	}

	fn set_size(&mut self, size: u32) {
		self.value &= !KmallocHeaderFlags::SIZE.bits();
		self.value |= size as u32;
	}

	fn size(&self) -> u32 {
		(self.value & KmallocHeaderFlags::SIZE.bits()) as u32
	}
}

pub unsafe fn kmalloc_init() {
	//core::ptr::write_bytes(start, 1, size as usize);

	KERNEL_HEAP_BREAK = KERNEL_HEAP_START;
	let header = KERNEL_HEAP_START as *mut KmallocHeader;
	(*header).set_used(false);
	(*header).set_size(KERNEL_HEAP_SIZE);

	println_serial!("Heap Start: {:#010X}", KERNEL_HEAP_START as usize);
	println_serial!("Heap End: {:#010X}", KERNEL_HEAP_END as usize);
}

/// Allocate a block of memory from the kernel heap.
///
/// This function searches the heap for a free block of memory of at least the requested size.
/// It ensures that even requests for very small memory sizes are handled efficiently by
/// setting a minimum allocation size.
///
/// # Arguments
///
/// * `size` - The size of the memory block to allocate. If the requested size is smaller than
///   the minimum allocation size, it will be increased to the minimum size.
///
/// # Returns
///
/// A pointer to the allocated memory block, or null if there is insufficient space or if the
/// allocation size exceeds the maximum allowable limit.
pub unsafe fn kmalloc(mut size: u32) -> Result<*mut u8, &'static str> {
	if size == 0 {
		return Err("kmalloc | Attempted to allocate zero bytes");
	}

	size += KMALLOC_HEADER_SIZE as u32;
	let remaining = size % MIN_ALLOCATION_SIZE;
	if remaining != 0 {
		size += MIN_ALLOCATION_SIZE - remaining;
	}

	if size > MAX_ALLOCATION_SIZE {
		return Err("kmalloc | Attempted to allocate invalid size");
	}

	let mut current = KERNEL_HEAP_START;
	while current < KERNEL_HEAP_END {
		let header = current as *mut KmallocHeader;
		if (*header).used() == false && (*header).size() >= size {
			if current.add(size as usize) > KERNEL_HEAP_BREAK {
				// TODO: Implement or call kbrk to increase heap size if necessary
				kbrk(size as isize); //TODO
			}
			let old_size = (*header).size();
			(*header).set_used(true);
			(*header).set_size(size);
			let next_header = current.add(size as usize) as *mut KmallocHeader;
			if old_size != size {
				(*next_header).set_used(false);
				(*next_header).set_size(old_size - size);
			}
			// let page_directory = &mut *PAGE_DIRECTORY.load(Ordering::SeqCst);
			// page_directory.map_range(
			// 	current as u32,
			// 	size,
			// 	PageDirectoryFlags::WRITABLE,
			// );
			return Ok(current.add(KMALLOC_HEADER_SIZE) as *mut u8);
		}
		current = current.add((*header).size() as usize);
	}
	Err("kmalloc | Insufficient space in kernel heap")
}

/// Free a previously allocated memory block.
///
/// # Arguments
///
/// * `ptr` - A pointer to the memory block to be freed.
pub unsafe fn kfree(ptr: *mut u8) {
	if ptr < KERNEL_HEAP_START || ptr >= KERNEL_HEAP_END {
		panic!(
			"kfree | Attempted to free invalid pointer: {:#010X}",
			ptr as usize
		);
	}

	let header_ptr = ptr.sub(KMALLOC_HEADER_SIZE) as *mut KmallocHeader;

	let mut current = KERNEL_HEAP_START;
	while current <= header_ptr as *mut u8 {
		let header = current as *mut KmallocHeader;
		if header == header_ptr {
			(*header_ptr).set_used(false);
			kdefrag();
			//free_empty_pages(header_ptr);
			return;
		}
		current = current.add((*header).size() as usize);
	}

	panic!(
		"kfree | Attempted to free invalid pointer: {:#010X}",
		ptr as usize
	);
}

fn free_empty_pages(ptr: *mut KmallocHeader) {
    unsafe {
        let block_size = (*ptr).size();
        let start_addr = ptr as usize;

        // Calculate the page-aligned start address
        let page_aligned_start = (start_addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        // Calculate the end address of the block
        let end_addr = start_addr + block_size as usize;

        // Calculate the start and end page numbers
        let start_page_number = page_aligned_start / PAGE_SIZE;
        let end_page_number = (end_addr + PAGE_SIZE - 1) / PAGE_SIZE;

        // Calculate the number of pages to unmap
        let num_pages = end_page_number - start_page_number;

        for i in 0..num_pages {
            let page_to_unmap = (start_page_number + i) * PAGE_SIZE;
            let page_directory = &mut *PAGE_DIRECTORY.load(Ordering::SeqCst);
            page_directory.unmap(page_to_unmap as u32);
        }
    }
}

pub unsafe fn kdefrag() {
	let mut header = KERNEL_HEAP_START as *mut KmallocHeader;

	while (header as *mut u8) < KERNEL_HEAP_END {
		let next_header = (header as *mut u8).add((*header).size() as usize) as *mut KmallocHeader;

		if (next_header as *mut u8) >= KERNEL_HEAP_END {
			break;
		}

		if !(*header).used() && !(*next_header).used() {
			let new_size = (*header).size() + (*next_header).size();
			(*header).set_size(new_size);
		} else {
			header = next_header;
		}
	}
}

/// Adjust the size of the kernel heap.
///
/// This function changes the size of the kernel heap by allocating or deallocating frames.
/// It handles the physical memory management by interfacing with the PMM.
///
/// # Arguments
///
/// * `byte` - The number of bytes to increase or decrease the heap by. Positive values increase
///   the heap size, while negative values decrease it.
///
/// # Returns
///
/// An `Option` containing the new break address if successful, or `None` if the operation fails.
fn kbrk(increment: isize) -> *mut u8 {
	unsafe {
		let new_break = KERNEL_HEAP_BREAK.offset(increment);
		if new_break > KERNEL_HEAP_END {
			panic!("Out of heap memory");
		}

		while KERNEL_HEAP_BREAK < new_break {
			let virtual_address = KERNEL_HEAP_BREAK as usize;

			// Check if the current virtual address already has a mapped frame
			let directory_index = virtual_address / (PAGE_SIZE * ENTRY_COUNT);
			let page_table_index = (virtual_address % (PAGE_SIZE * ENTRY_COUNT)) / PAGE_SIZE;

			let page_directory_ptr = PAGE_DIRECTORY.load(Ordering::SeqCst);
			let mut page_table = (*page_directory_ptr).entries[directory_index].get_page_table();

			// If the page table does not exist, create it
			if page_table.is_none() {
				let new_table_frame = PMM
					.lock()
					.allocate_frame()
					.expect("Out of physical memory for page table");
				(*page_directory_ptr).entries[directory_index].set(
					new_table_frame,
					PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
				);
				page_table = (*page_directory_ptr).entries[directory_index].get_page_table();
			}

			// Check if the specific page within the table is mapped
			if let Some(ref mut page_table) = page_table {
				if page_table.entries[page_table_index].is_unused() {
					let page = PMM.lock().allocate_frame().expect("Out of physical memory");
					let page_directory = &mut *PAGE_DIRECTORY.load(Ordering::SeqCst);
					page_directory.map(virtual_address as u32, page, PageDirectoryFlags::WRITABLE);
				}
			}

			// Increment KERNEL_HEAP_BREAK by one page size until it reaches or surpasses new_break
			KERNEL_HEAP_BREAK = KERNEL_HEAP_BREAK.offset(PAGE_SIZE as isize);

			// Stop if we've reached or surpassed the new break point
			if KERNEL_HEAP_BREAK >= new_break {
				break;
			}
		}

		KERNEL_HEAP_BREAK
	}
}

/// Get the size of a memory block allocated by kmalloc.
///
/// # Arguments
///
/// * `ptr` - A pointer to the allocated memory block.
///
/// # Returns
///
/// The size of the allocated memory block.
pub unsafe fn ksize(ptr: *mut u8) -> Option<u32> {
	if ptr.is_null() || ptr < KERNEL_HEAP_START || ptr >= KERNEL_HEAP_END {
		panic!(
			"ksize | Attempted to get size of invalid pointer: {:#010X}",
			ptr as usize
		);
	}

	let header_ptr = ptr.sub(KMALLOC_HEADER_SIZE) as *mut KmallocHeader;

	if (*header_ptr).used() == false {
		panic!(
			"ksize | Attempted to get size of unallocated pointer: {:#010X}",
			ptr as usize
		);
	}

	Some((*header_ptr).size())
}

pub fn kmalloc_tester() {
	unsafe {
		//kmalloc_init(0x600000 as *mut u8, 0x10000 as u32);

		let a = kmalloc(8).unwrap();
		//crate::memory::page_directory::init_pages();
		let b = kmalloc(253).unwrap();
		let c = kmalloc(1020).unwrap();

		kprint_heap();
		kfree(b);
		let d = kmalloc(50).unwrap();
		kprint_heap();
		kfree(c);
		kprint_heap();
		let y = kmalloc(3000).unwrap();
		for i in 0..3000 {
			*y.add(i) = 0x42;
		}
		let z = kmalloc(54000).unwrap();
		*z = 0x55;
		kfree(y);
		kprint_heap();
		kfree(a);
		kprint_heap();
		kfree(d);
		kprint_heap();
		kfree(z);
		kprint_heap();

		//let d = kmalloc(12).unwrap();
		//println!("d: {:#010X}", d as usize);
	}
}

pub fn kprint_heap() {
	unsafe {
		let mut current = KERNEL_HEAP_START;
		println_serial!("");
		println_serial!("Heap Start: {:#010X}", KERNEL_HEAP_START as usize);
		println_serial!("Heap End: {:#010X}", KERNEL_HEAP_END as usize);
		println_serial!("Kernel Heap Break: {:#010X}", KERNEL_HEAP_BREAK as usize);

		while current < KERNEL_HEAP_END {
			let header = current as *mut KmallocHeader;
			println_serial!(
				"{x:08X} | size: {s:08X} | used: {u}",
				x = current as usize,
				s = (*header).size(),
				u = (*header).used()
			);
			current = current.add((*header).size() as usize);
		}
	}
}
