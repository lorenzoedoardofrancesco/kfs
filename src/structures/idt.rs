//! # Interrupt Handling Module (IDT)
//!
//! This module provides the structures and functions necessary to handle
//! interrupts in a low-level context of an operating system kernel.
//! The module is responsible for setting up the Interrupt Descriptor Table (IDT)
//! and loading it into the CPU.
//!
//! ## Overview
//!
//!  The IDT is used by the CPU to determine the correct response to various
//!  interrupt and exception conditions. The IDT is loaded into the CPU's IDTR
//! register using the `lidt` instruction. It includes definitions for standard
//! interrupts like the divide by zero exception, page fault exception, and
//! timer interrupt. The IDT also includes definitions for hardware interrupts
//! like the keyboard interrupt and RTC interrupt.

use crate::exceptions::interrupts::InterruptIndex;
use crate::exceptions::interrupts::{
	alignment_check, bound_range_exceeded, breakpoint, coprocessor_not_available,
	coprocessor_segment_overrun, debug, divide_by_zero, double_fault, general_protection_fault,
	invalid_opcode, invalid_task_state_segment, keyboard_interrupt, machine_check, math_fault,
	non_maskable_interrupt, overflow, page_fault, reserved, segment_not_present,
	simd_floating_point_exception, stack_fault, timer_interrupt, virtualization_exception,
};
use core::arch::asm;
use lazy_static::lazy_static;

/// Represents an Interrupt Descriptor Table (IDT) entry.
///
/// The IDT entry is used by the CPU to determine the correct response
/// to various interrupt and exception conditions.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IdtDescriptor {
	offset_low: u16,
	selector: u16,
	reserved: u8,
	type_attributes: u8,
	offset_high: u16,
}

impl IdtDescriptor {
	/// Creates a new IDT entry.
	///
	/// # Arguments
	///
	/// * `offset` - The address of the interrupt service routine.
	/// * `selector` - The code segment selector.
	/// * `type_attributes` - Type and attributes of the interrupt gate.
	///
	fn new(offset: u32, selector: u16, type_attributes: u8) -> IdtDescriptor {
		IdtDescriptor {
			offset_low: (offset & 0xffff) as u16,
			selector: selector,
			reserved: 0,
			type_attributes,
			offset_high: ((offset >> 16) & 0xffff) as u16,
		}
	}
}

// Static declarations for interrupt handlers.
// These handlers are set to manage specific CPU interrupts and exceptions.

/// Handler for the 'Divide by Zero' exception.
static DIVIDE_BY_ZERO: extern "C" fn() = handler!(divide_by_zero);

/// Handler for the 'Debug' exception.
static DEBUGG: extern "C" fn() = handler!(debug);

/// Handler for the 'Non-Maskable Interrupt' (NMI).
static NON_MASKABLE_INTERRUPT: extern "C" fn() = handler!(non_maskable_interrupt);

/// Handler for the 'Breakpoint' exception.
static BREAKPOINT: extern "C" fn() = handler!(breakpoint);

/// Handler for the 'Overflow' exception.
static OVERFLOW: extern "C" fn() = handler!(overflow);

/// Handler for the 'Bound Range Exceeded' exception.
static BOUND_RANGE_EXCEEDED: extern "C" fn() = handler!(bound_range_exceeded);

/// Handler for the 'Invalid Opcode' exception.
static INVALID_OPCODE: extern "C" fn() = handler!(invalid_opcode);

/// Handler for the 'Coprocessor Not Available' exception.
static COPROCESSOR_NOT_AVAILABLE: extern "C" fn() = handler!(coprocessor_not_available);

/// Handler for the 'Double Fault' exception.
static DOUBLE_FAULT: extern "C" fn() = handler!(double_fault);

/// Handler for the 'Coprocessor Segment Overrun' exception.
static COPROCESSOR_SEGMENT_OVERRUN: extern "C" fn() = handler!(coprocessor_segment_overrun);

/// Handler for the 'Invalid Task State Segment' exception.
static INVALID_TASK_STATE_SEGMENT: extern "C" fn() = handler!(invalid_task_state_segment);

/// Handler for the 'Segment Not Present' exception.
static SEGMENT_NOT_PRESENT: extern "C" fn() = handler!(segment_not_present);

/// Handler for the 'Stack Fault' exception.
static STACK_FAULT: extern "C" fn() = handler!(stack_fault);

/// Handler for the 'General Protection Fault' exception.
static GENERAL_PROTECTION_FAULT: extern "C" fn() = handler!(general_protection_fault);

/// Handler for the 'Page Fault' exception.
static PAGE_FAULT: extern "C" fn() = handler!(page_fault);

/// Reserved handler.
static RESERVED: extern "C" fn() = handler!(reserved);

/// Handler for the 'Math Fault' exception.
static MATH_FAULT: extern "C" fn() = handler!(math_fault);

/// Handler for the 'Alignment Check' exception.
static ALIGNMENT_CHECK: extern "C" fn() = handler!(alignment_check);

/// Handler for the 'Machine Check' exception.
static MACHINE_CHECK: extern "C" fn() = handler!(machine_check);

/// Handler for the 'SIMD Floating Point Exception'.
static SIMD_FLOATING_POINT_EXCEPTION: extern "C" fn() = handler!(simd_floating_point_exception);

/// Handler for the 'Virtualization Exception'.
static VIRTUALIZATION_EXCEPTION: extern "C" fn() = handler!(virtualization_exception);

/// Handler for the 'Timer Interrupt'.
static TIMER_INTERRUPT: extern "C" fn() = handler!(timer_interrupt);

/// Handler for the 'Keyboard Interrupt'.
static KEYBOARD_INTERRUPT: extern "C" fn() = handler!(keyboard_interrupt);

lazy_static! {
	/// Lazy static initialization of the Interrupt Descriptor Table (IDT).
	///
	/// This block sets up the IDT with predefined entries for standard interrupts
	#[link_section = ".idt"]
	static ref IDT: [IdtDescriptor; 256] = {
		let mut idt = [IdtDescriptor::new(0, 0, 0); 256];

		idt[0] = IdtDescriptor::new(DIVIDE_BY_ZERO as u32, 0x08, 0x8e);
		idt[1] = IdtDescriptor::new(DEBUGG as u32, 0x08, 0x8e);
		idt[2] = IdtDescriptor::new(NON_MASKABLE_INTERRUPT as u32, 0x08, 0x8e);
		idt[3] = IdtDescriptor::new(BREAKPOINT as u32, 0x08, 0x8e);
		idt[4] = IdtDescriptor::new(OVERFLOW as u32, 0x08, 0x8e);
		idt[5] = IdtDescriptor::new(BOUND_RANGE_EXCEEDED as u32, 0x08, 0x8e);
		idt[6] = IdtDescriptor::new(INVALID_OPCODE as u32, 0x08, 0x8e);
		idt[7] = IdtDescriptor::new(COPROCESSOR_NOT_AVAILABLE as u32, 0x08, 0x8e);
		idt[8] = IdtDescriptor::new(DOUBLE_FAULT as u32, 0x08, 0x8e);
		idt[9] = IdtDescriptor::new(COPROCESSOR_SEGMENT_OVERRUN as u32, 0x08, 0x8e);
		idt[10] = IdtDescriptor::new(INVALID_TASK_STATE_SEGMENT as u32, 0x08, 0x8e);
		idt[11] = IdtDescriptor::new(SEGMENT_NOT_PRESENT as u32, 0x08, 0x8e);
		idt[12] = IdtDescriptor::new(STACK_FAULT as u32, 0x08, 0x8e);
		idt[13] = IdtDescriptor::new(GENERAL_PROTECTION_FAULT as u32, 0x08, 0x8e);
		idt[14] = IdtDescriptor::new(PAGE_FAULT as u32, 0x08, 0x8e);
		idt[15] = IdtDescriptor::new(RESERVED as u32, 0x08, 0x8e);
		idt[16] = IdtDescriptor::new(MATH_FAULT as u32, 0x08, 0x8e);
		idt[17] = IdtDescriptor::new(ALIGNMENT_CHECK as u32, 0x08, 0x8e);
		idt[18] = IdtDescriptor::new(MACHINE_CHECK as u32, 0x08, 0x8e);
		idt[19] = IdtDescriptor::new(SIMD_FLOATING_POINT_EXCEPTION as u32, 0x08, 0x8e);
		idt[20] = IdtDescriptor::new(VIRTUALIZATION_EXCEPTION as u32, 0x08, 0x8e);
		idt[InterruptIndex::Timer.as_usize()] = IdtDescriptor::new(TIMER_INTERRUPT as u32, 0x08, 0x8e);
		idt[InterruptIndex::Keyboard.as_usize()] = IdtDescriptor::new(KEYBOARD_INTERRUPT as u32, 0x08, 0x8e);
		idt
	};
}

/// Represents the register structure used for loading the IDT.
///
/// This structure is required for the `lidt` instruction which loads
/// the address and size of the IDT into the CPU's IDT register.
#[repr(C, packed)]
struct IdtRegister {
	size: u16,
	offset: u32,
}

/// Initializes and loads the Interrupt Descriptor Table (IDT).
///
/// This function constructs the IDT register structure and uses inline
/// assembly to load it into the CPU, effectively setting up the system
/// to handle interrupts and exceptions as defined in the IDT.
///
/// # Safety
///
/// This function is marked unsafe as it involves low-level operations
/// that manipulate CPU state.
pub fn init() {
	unsafe {
		let idt_register = IdtRegister {
			size: (core::mem::size_of::<[IdtDescriptor; 256]>() - 1) as u16,
			offset: IDT.as_ptr() as u32,
		};

		asm!("lidt [{}]", in(reg) &idt_register, options(readonly, nostack, preserves_flags));

		println_serial!("IDT successfully loaded");
	}
}
