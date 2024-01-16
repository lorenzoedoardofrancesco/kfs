//! # Interrupt Handling Module
//!
//! Provides functionality for setting up and handling interrupts in an x86 system. This module includes
//! the initialization of the Programmable Interrupt Controller (PIC), definitions of interrupt handler functions,
//! and utilities for enabling and disabling interrupts. The module plays a crucial role in the system's
//! response to hardware and software interrupts.
use crate::utils::debug::LogLevel;
use crate::utils::io::inb;
use crate::{
	exceptions::{
		keyboard::{BUFFER_HEAD, KEYBOARD_INTERRUPT_RECEIVED, SCANCODE_BUFFER},
		pic8259::ChainedPics,
	},
	memory::{
		page_directory::{PAGE_SIZE, PAGE_TABLES_ADDR},
		page_table::PageTable,
		page_table_entry::PageTableFlags,
		physical_memory_managment::PMM,
	},
};
use core::arch::asm;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

use super::panic::handle_panic;

pub static TICKS: AtomicU32 = AtomicU32::new(0);

pub const PIC_1_OFFSET: u8 = 32;

/// Global instance of chained PICs.
pub static PICS: Mutex<ChainedPics> =
	Mutex::new(unsafe { ChainedPics::new_contiguous(PIC_1_OFFSET) });

/// Enumeration of interrupt indexes.
///
/// Represents various interrupt lines corresponding to different hardware and software interrupts.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(u8)]
pub enum InterruptIndex {
	Timer = PIC_1_OFFSET,
	Keyboard,
	Cascade,
	Com2,
	Com1,
	Lpt2,
	Floppy,
	Lpt1,
	Rtc,
	Free1,
	Free2,
	Free3,
	Ps2Mouse,
	PrimaryAtaHardDisk,
	SecondaryAtaHardDisk,
}

impl InterruptIndex {
	pub fn as_u8(self) -> u8 {
		self as u8
	}

	pub fn as_usize(self) -> usize {
		usize::from(self.as_u8())
	}
}

/// Structure representing an interrupt stack frame.
///
/// This structure is pushed onto the stack by the CPU on an interrupt.
/// It contains the state of the CPU at the time of the interrupt.
/// The structure is used by the interrupt handler functions to determine
/// the cause of the interrupt and to handle it appropriately.
#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
	// The following registers are pushed by the CPU automatically
	pub eip: u32,    // Instruction pointer
	pub cs: u32,     // Code segment
	pub eflags: u32, // CPU flags

	// These are only present if the interrupt involved a privilege level change
	pub esp: u32, // Stack pointer (optional)
	pub ss: u32,  // Stack segment (optional)
}

/// Handler functions for various interrupts.
///
/// Each of these functions handles a specific type of interrupt, such as
/// divide by zero, page fault, keyboard input, etc.
///
/// The functions print a message and the state of the stack frame at the time
/// of the interrupt. Not yet implemented.

pub extern "C" fn divide_by_zero(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Divide By Zero", Some(stack_frame));
}

pub extern "C" fn debug(stack_frame: &mut InterruptStackFrame) {
	log!(LogLevel::Info, "EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

pub extern "C" fn non_maskable_interrupt(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}",
		stack_frame
	);
}

pub extern "C" fn breakpoint(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: BREAKPOINT at {:#x}\n{:#?}",
		stack_frame.eip,
		stack_frame
	);
}

pub extern "C" fn overflow(stack_frame: &mut InterruptStackFrame) {
	log!(LogLevel::Info, "EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

pub extern "C" fn bound_range_exceeded(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}",
		stack_frame
	);
}

pub extern "C" fn invalid_opcode(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Invalid Opcode", Some(stack_frame));
}

pub extern "C" fn coprocessor_not_available(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: COPROCESSOR NOT AVAILABLE\n{:#?}",
		stack_frame
	);
}

pub extern "C" fn double_fault(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Double Fault", Some(stack_frame));
}

pub extern "C" fn coprocessor_segment_overrun(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: COPROCESSOR SEGMENT OVERRUN\n{:#?}",
		stack_frame
	);
}

pub extern "C" fn invalid_task_state_segment(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Invalid Task State Segment", Some(stack_frame));
}

pub extern "C" fn segment_not_present(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Segment Not Present", Some(stack_frame));
}

pub extern "C" fn stack_fault(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Stack Fault", Some(stack_frame));
}

pub extern "C" fn general_protection_fault(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"General Protection Fault", Some(stack_frame));
}

#[no_mangle]
pub extern "C" fn page_fault(_stack_frame: &mut InterruptStackFrame, error_code: u32) {
	let faulting_address: u32;
	unsafe {
		asm!("mov {}, cr2", out(reg) faulting_address, options(nostack, preserves_flags));
	}

	let present = error_code & 0x1 != 0; // False if the page was not present
	let write = error_code & 0x2 != 0; // True if it was a write operation, false for read
	let user = error_code & 0x4 != 0; // True if the fault occurred in user mode
	let reserved = error_code & 0x8 != 0; // True if caused by reserved bit violation
	let instruction_fetch = error_code & 0x10 != 0; // True if caused by an instruction fetch

	log!(
		LogLevel::Alert,
		"Page Fault at address {:#x}\tError Code: {:#x}",
		faulting_address,
		error_code
	);

	if !present {
		log!(LogLevel::Info, "Page not present");
		handle_not_present_page_fault(faulting_address as usize);
	} else {
		if write {
			log!(LogLevel::Error, "Attempted to write to a read-only page");
		} else if user {
			log!(
				LogLevel::Error,
				"Attempted to access kernel page from user mode"
			);
		} else if reserved {
			log!(
				LogLevel::Panic,
				"Reserved bit violation in page table entry"
			);
			panic!("Kernel panic: Reserved bit violation");
		} else if instruction_fetch {
			log!(
				LogLevel::Panic,
				"Instruction fetch from a non-executable page"
			);
			panic!("Kernel panic: Attempted to execute non-executable memory");
		} else {
			log!(LogLevel::Error, "Unknown page fault");
		}
	}

	// Additional handling can be added here as required
}

fn handle_not_present_page_fault(faulting_address: usize) {
	// Calculate the page directory index and page table index.
	let pd_index = faulting_address >> 22;
	let pt_index = (faulting_address >> 12) & 0x3FF;

	println_serial!("Page directory index: {}", pd_index);
	println_serial!("Page table index: {}", pt_index);

	// Access the page table
	let page_table_addr = unsafe { PAGE_TABLES_ADDR + (pd_index * PAGE_SIZE) as u32 };
	let page_table = unsafe { &mut *(page_table_addr as *mut PageTable) };

	// Allocate a new frame for the page
	println_serial!("Page table address: {:#x}, page_table ", page_table_addr);
	println_serial!("Page table address: {:#x}, page_table ", page_table_addr);
	let frame = PMM.lock().allocate_frame();
	match frame {
		Ok(frame) => {
			println_serial!("Allocated frame: {:#x}", frame);

			// Update the page table entry
			let page_table_entry = &mut page_table.entries[pt_index];
			page_table_entry
				.set_frame_address(frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
			println_serial!("Updated page table entry: {:#x}", page_table_entry.value());

			// Optionally, flush the TLB for the updated address here
			unsafe {
				let cr3: u32;
				asm!("mov {}, cr3", out(reg) cr3);
				asm!("mov cr3, {}", in(reg) cr3);
			}
		}
		Err(e) => {
			log!(
				LogLevel::Error,
				"Failed to allocate frame for page table entry: {}",
				e
			);
			return;
		}
	}
}

// fn handle_not_present_page_fault(faulting_address: usize, write: bool, user: bool) {
// 	let page_directory: &mut PageDirectory =
// 		unsafe { &mut *PAGE_DIRECTORY.load(Ordering::Relaxed) };

// 	// Calculate the page directory index and page table index.
// 	let pd_index = faulting_address >> 22;
// 	let pt_index = (faulting_address >> 12) & 0x3FF;

// 	println_serial!("Page directory index: {}", pd_index);
// 	println_serial!("Page table index: {}", pt_index);

// 	// Check if the page table exists.
// 	let page_table_addr = unsafe { PAGE_TABLES_ADDR + (pd_index * PAGE_SIZE) as u32 };
// 	let page_table = unsafe { &mut *(page_table_addr as *mut PageTable) };

// 	// Allocate a new frame for the page.
// 	let pmm = unsafe { &mut *(PMM_ADDRESS as *mut PhysicalMemoryManager) };
// 	let frame: u32 = pmm.allocate_frame().unwrap();
// 	println_serial!("MIAOOO");
// 	println_serial!("Page table address: {:#x}", page_table_addr);
// 	let page_table_entry = (page_table_addr as usize + (pt_index * 4)) as *mut PageTableEntry;
// 	println_serial!("Page table entry: {:#x}", page_table_entry as usize);
// 	unsafe {
// 		(*page_table_entry).value =
// 			frame | PageTableFlags::PRESENT.bits() | PageTableFlags::WRITABLE.bits()
// 	};
// 	println_serial!("Page table entry value: {:#x}", unsafe {
// 		(*page_table_entry).value
// 	});
// 	//page_table.add_entry(pt_index, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
// }

pub extern "C" fn reserved(stack_frame: &mut InterruptStackFrame) {
	log!(LogLevel::Info, "EXCEPTION: RESERVED\n{:#?}", stack_frame);
}

pub extern "C" fn math_fault(stack_frame: &mut InterruptStackFrame) {
	log!(LogLevel::Info, "EXCEPTION: MATH FAULT\n{:#?}", stack_frame);
}

pub extern "C" fn alignment_check(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Alignment Check", Some(stack_frame));
}

pub extern "C" fn machine_check(stack_frame: &mut InterruptStackFrame) {
	handle_panic(&"Machine Check", Some(stack_frame));
}

pub extern "C" fn simd_floating_point_exception(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: SIMD FLOATING POINT EXCEPTION\n{:#?}",
		stack_frame
	);
}

pub extern "C" fn virtualization_exception(stack_frame: &mut InterruptStackFrame) {
	log!(
		LogLevel::Info,
		"EXCEPTION: VIRTUALIZATION EXCEPTION\n{:#?}",
		stack_frame
	);
}

pub fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
	unsafe {
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
	TICKS.fetch_add(1, Ordering::SeqCst);
}

pub fn keyboard_interrupt(_stack_frame: &mut InterruptStackFrame) {
	let scancode: u8 = unsafe { inb(0x60) };

	unsafe {
		SCANCODE_BUFFER[BUFFER_HEAD] = scancode;
		BUFFER_HEAD = (BUFFER_HEAD + 1) % SCANCODE_BUFFER.len();
		KEYBOARD_INTERRUPT_RECEIVED.store(true, Ordering::SeqCst);
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
	}
}

pub fn syscall_interrupt(_stack_frame: &mut InterruptStackFrame) {
	use crate::exceptions::syscalls::{syscall, GeneralRegs};

	let mut registers = GeneralRegs {
		eax: 0,
		ebx: 0,
		ecx: 0,
		edx: 0,
		esi: 0,
		edi: 0,
		ebp: 0,
	};

	unsafe {
		asm!(
			// Save registers into the struct fields
			"mov [{}], eax",
			"mov [{}], ebx",
			"mov [{}], ecx",
			"mov [{}], edx",
			"mov [{}], esi",
			"mov [{}], edi",
			"mov [{}], ebp",
			// Pointers to each field of the `registers` struct
			in(reg) &mut registers.eax,
			in(reg) &mut registers.ebx,
			in(reg) &mut registers.ecx,
			in(reg) &mut registers.edx,
			in(reg) &mut registers.esi,
			in(reg) &mut registers.edi,
			in(reg) &mut registers.ebp,
			options(preserves_flags, nostack)
		);
	}

	// Call the syscall function with the registers
	syscall(&mut registers);

	unsafe {
		asm!(
			// Restore the register values from the struct fields
			"mov eax, [{}]",
			"mov ebx, [{}]",
			"mov ecx, [{}]",
			"mov edx, [{}]",
			"mov esi, [{}]",
			"mov edi, [{}]",
			"mov ebp, [{}]",
			// Pointers to each field of the `registers` struct
			in(reg) &registers.eax,
			in(reg) &registers.ebx,
			in(reg) &registers.ecx,
			in(reg) &registers.edx,
			in(reg) &registers.esi,
			in(reg) &registers.edi,
			in(reg) &registers.ebp,
			options(preserves_flags, nostack)
		);
	}
}

/// Initializes the interrupt handlers.
///
/// This function sets up the PICs and enables interrupts in the system.
pub fn init() {
	unsafe {
		PICS.lock().initialize();
	}
	log!(
		LogLevel::Info,
		"PIC successfully initialized (Master: {:#x}, Slave: {:#x})",
		PIC_1_OFFSET,
		PIC_1_OFFSET + 8
	);
	enable();
	log!(LogLevel::Info, "Interrupts successfully enabled");
}

/// Enables interrupts on the CPU.
///
/// This function enables interrupts on the CPU by setting the interrupt flag in the CPU's flags register.
pub fn enable() {
	unsafe {
		asm!("sti", options(preserves_flags, nostack));
	}
}

/// Disables interrupts on the CPU.
///
/// This function disables interrupts on the CPU by clearing the interrupt flag in the CPU's flags register.
pub fn disable() {
	unsafe {
		asm!("cli", options(preserves_flags, nostack));
	}
}
