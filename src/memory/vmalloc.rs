//! Implementation of vmalloc, vfree, vsbrk, and vsize for memory allocation.
//! Implementation of vmalloc for virtual memory allocation.
//! vmalloc manages memory allocations that can span multiple pages and are not necessarily contiguous in physical memory.



use crate::memory::{ page_directory::PAGE_SIZE, physical_memory_managment::PMM};

/// Allocates a block of memory in the virtual address space.
/// 
/// # Arguments
/// 
/// * `size` - The size of the memory block to allocate.
/// 
/// # Returns
/// 
/// A pointer to the allocated memory block, or null if there is insufficient space.
pub unsafe fn vmalloc(size: usize) -> *mut u8 {
    let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

    // Implement the logic to find and allocate sufficient contiguous virtual pages.
    let virtual_address = allocate_virtual_pages(num_pages);
    if virtual_address.is_null() {
        return core::ptr::null_mut();
    }

    // Using the physical memory manager from pmm.rs to allocate frames
    for i in 0..num_pages {
        let frame = PMM.lock().allocate_frame();
        if frame.is_err() {
            // Handle failure to allocate a physical frame.
            return core::ptr::null_mut();
        }

        let physical_address = frame.unwrap();
        map_virtual_to_physical(virtual_address.add(i * PAGE_SIZE), physical_address as usize);
    }

    virtual_address
}

/// Allocates contiguous virtual pages.
/// 
/// # Arguments
/// 
/// * `num_pages` - The number of contiguous pages to allocate.
/// 
/// # Returns
/// 
/// A pointer to the start of the allocated virtual pages, or null if allocation fails.
unsafe fn allocate_virtual_pages(num_pages: usize) -> *mut u8 {

    let address = PMM.lock().allocate_multiple_frames(num_pages as u32);
    core::ptr::null_mut() // Placeholder
}

/// Maps a range of virtual addresses to a physical address.
/// 
/// # Arguments
/// 
/// * `virtual_address` - The starting virtual address to map.
/// * `physical_address` - The physical address to map to.
unsafe fn map_virtual_to_physical(virtual_address: *mut u8, physical_address: usize) {
    // Implement the logic to map a virtual address to a physical address.
}
