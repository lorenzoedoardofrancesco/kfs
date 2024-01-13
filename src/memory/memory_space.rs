// use crate::memory::{
// 	page_directory::{enable_paging, PageDirectory, PageDirectoryEntry, ENTRY_COUNT, PAGE_DIRECTORY, PAGE_TABLES, PageDirectoryFlags},
// 	page_table_entry::{PageTableEntry, PageTableFlags},
//     physical_memory_managment::{ KERNEL_SPACE_START, KERNEL_SPACE_END, USER_SPACE_START, USER_SPACE_END },
// };
// use core::sync::atomic::{AtomicPtr, Ordering};


// pub fn init_pages() {
//     unsafe {
//         let directory = &mut *PAGE_DIRECTORY.load(Ordering::Relaxed);
//         let tables = &mut *PAGE_TABLES.load(Ordering::Relaxed);

//         for (i, table) in tables.iter_mut().enumerate() {
//             let table_phys_addr = table.entries.as_ptr() as u32;

//             for (j, entry) in table.entries.iter_mut().enumerate() {
//                 let virt: u32 = ((i << 22) | (j << 12)) as u32;
//                 let phys: u32 = virt as u32;
//                 entry.set_frame_address(phys);

//                 // Determine if the address is within kernel or user space and set attributes accordingly
//                 if virt >= KERNEL_SPACE_START && virt < KERNEL_SPACE_END {
//                     // Kernel space - typically writable and present, but not user accessible
//                     entry.add_attribute(PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
//                 } else if virt >= USER_SPACE_START && virt < USER_SPACE_END {
//                     // User space - writable, present, and user accessible
//                     entry.add_attribute(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER);
//                 } else {
//                     // Not mapped
//                     entry.remove_attribute(PageTableFlags::PRESENT);
//                 }
//             }

//             // Add the page table to the directory
//             directory.add_entry(
// 				i,
				
//         }
//     }

//     enable_paging();
// }

