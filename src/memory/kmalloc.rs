//! Implementation of kmalloc, kfree, ksbrk, and ksize for memory allocation.
//! kmalloc and kfree manage memory in a simple linear heap.
//! ksbrk adjusts the heap size, and ksize returns the size of an allocated block.

use crate::memory::{ page_directory::PAGE_SIZE, physical_memory_managment::PMM};

static mut HEAP_START: *mut u8 = 0 as *mut u8;
static mut HEAP_END: *mut u8 = 0 as *mut u8;

#[repr(C, packed)]
struct KmallocHeader {
    size: usize,
    used: bool,
}

pub unsafe fn kmalloc_init(start: *mut u8, size: usize) {
    HEAP_START = start;
    HEAP_END = start.add(size);
}

/// Allocate a block of memory from the kernel heap.
/// 
/// # Arguments
/// 
/// * `size` - The size of the memory block to allocate.
/// 
/// # Returns
/// 
/// A pointer to the allocated memory block, or null if there is insufficient space.
pub unsafe fn kmalloc(size: usize) -> *mut u8 {
    let mut current = HEAP_START;
    while current < HEAP_END {
        let header = current as *mut KmallocHeader;
        if (*header).used == false && (*header).size >= size {
            (*header).used = true;
            return current.add(core::mem::size_of::<KmallocHeader>());
        }
        current = current.add((*header).size);
    }
    core::ptr::null_mut() // Out of memory
}

/// Free a previously allocated memory block.
/// 
/// # Arguments
/// 
/// * `ptr` - A pointer to the memory block to be freed.
pub unsafe fn kfree(ptr: *mut u8) {
    let header = (ptr as *mut u8).sub(core::mem::size_of::<KmallocHeader>()) as *mut KmallocHeader;
    (*header).used = false;
}

/// Adjust the size of the kernel heap.
/// 
/// # Arguments
/// 
/// * `n` - The number of bytes to increase or decrease the heap by.
pub unsafe fn kbrk(increment: isize) -> *mut u8 {
    static mut KERNEL_HEAP_BREAK: *mut u8 = unsafe { HEAP_START };

    // If increment is 0, simply return the current break
    if increment == 0 {
        return KERNEL_HEAP_BREAK;
    }

    let new_break = KERNEL_HEAP_BREAK.offset(increment);
    if new_break >= HEAP_START && new_break < HEAP_END {
        let old_break = KERNEL_HEAP_BREAK;
        KERNEL_HEAP_BREAK = new_break;
        old_break // Return the old break value
    } else {
        // Return error if the new break is outside the heap bounds
        -1_isize as *mut u8
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
pub unsafe fn ksize(ptr: *mut u8) -> usize {
    if ptr.is_null() {
        return 0; // Safety check: return 0 if the pointer is null
    }

    // Calculate the address of the header by subtracting the size of the header from the pointer
    let header_ptr = ptr.sub(core::mem::size_of::<KmallocHeader>()) as *const KmallocHeader;

    (*header_ptr).size
}
