//! # Interrupt Handling Module
//!
//! Provides functionality for setting up and handling interrupts in an x86 system. This module includes
//! the initialization of the Programmable Interrupt Controller (PIC), definitions of interrupt handler functions,
//! and utilities for enabling and disabling interrupts. The module plays a crucial role in the system's
//! response to hardware and software interrupts.
use crate::exceptions::keyboard::{BUFFER_HEAD, KEYBOARD_INTERRUPT_RECEIVED, SCANCODE_BUFFER};
use crate::exceptions::pic8259::ChainedPics;
use crate::utils::debug::LogLevel;
use crate::utils::io::inb;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

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
	instruction_pointer: u32,
	code_segment: u32,
	cpu_flags: u32,
	stack_pointer: u32,
	stack_segment: u32,
	eax: u32,
	ebx: u32,
	ecx: u32,
	edx: u32,
	esi: u32,
	edi: u32,
	ebp: u32,
}

/// Handler functions for various interrupts.
///
/// Each of these functions handles a specific type of interrupt, such as
/// divide by zero, page fault, keyboard input, etc.
///
/// The functions print a message and the state of the stack frame at the time
/// of the interrupt. Not yet implemented.
pub extern "C" fn divide_by_zero(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: DIVIDE BY ZERO\n{:#x?}", _stack_frame);
}

pub extern "C" fn debug(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: DEBUG\n{:#x?}", _stack_frame);
}

pub extern "C" fn non_maskable_interrupt(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#x?}", _stack_frame);
}

pub extern "C" fn breakpoint(_stack_frame: &mut InterruptStackFrame) {
	let stack_frame = &mut *_stack_frame;
	println!(
		"EXCEPTION: BREAKPOINT at {:#x}\n{:#x?}",
		stack_frame.instruction_pointer, stack_frame
	);
}

pub fn overflow(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: OVERFLOW\n{:#x?}", _stack_frame);
}

pub fn bound_range_exceeded(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#x?}", _stack_frame);
}

pub fn invalid_opcode(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: INVALID OPCODE\n{:#x?}", _stack_frame);
}

pub fn coprocessor_not_available(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: COPROCESSOR NOT AVAILABLE\n{:#x?}", _stack_frame);
}

pub fn double_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: DOUBLE FAULT\n{:#x?}", _stack_frame);
}

pub fn coprocessor_segment_overrun(_stack_frame: &mut InterruptStackFrame) {
	println!(
		"EXCEPTION: COPROCESSOR SEGMENT OVERRUN\n{:#x?}",
		_stack_frame
	);
}

pub fn invalid_task_state_segment(_stack_frame: &mut InterruptStackFrame) {
	println!(
		"EXCEPTION: INVALID TASK STATE SEGMENT\n{:#x?}",
		_stack_frame
	);
}

pub fn segment_not_present(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: SEGMENT NOT PRESENT\n{:#x?}", _stack_frame);
}

pub fn stack_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: STACK FAULT\n{:#x?}", _stack_frame);
}

pub fn general_protection_fault(stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#x?}", stack_frame);
}

pub fn page_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: PAGE FAULT\n{:#x?}", _stack_frame);
}

pub fn reserved(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: RESERVED\n{:#x?}", _stack_frame);
}

pub fn math_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: MATH FAULT\n{:#x?}", _stack_frame);
}

pub fn alignment_check(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: ALIGNMENT CHECK\n{:#x?}", _stack_frame);
}

pub fn machine_check(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: MACHINE CHECK\n{:#x?}", _stack_frame);
}

pub fn simd_floating_point_exception(_stack_frame: &mut InterruptStackFrame) {
	println!(
		"EXCEPTION: SIMD FLOATING POINT EXCEPTION\n{:#x?}",
		_stack_frame
	);
}

pub fn virtualization_exception(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: VIRTUALIZATION EXCEPTION\n{:#x?}", _stack_frame);
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
		eax: _stack_frame.eax,
		ebx: _stack_frame.ebx,
		ecx: _stack_frame.ecx,
		edx: _stack_frame.edx,
		esi: _stack_frame.esi,
		edi: _stack_frame.edi,
		ebp: _stack_frame.ebp,
	};

	syscall(&mut registers);

	_stack_frame.eax = registers.eax;
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
	use core::arch::asm;
	unsafe {
		asm!("sti", options(preserves_flags, nostack));
	}
}

/// Disables interrupts on the CPU.
///
/// This function disables interrupts on the CPU by clearing the interrupt flag in the CPU's flags register.
pub fn disable() {
	use core::arch::asm;
	unsafe {
		asm!("cli", options(preserves_flags, nostack));
	}
}
