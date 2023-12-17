use crate::io::inb;
use crate::pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;

pub static PICS: Mutex<ChainedPics> =
	Mutex::new(unsafe { ChainedPics::new_contiguous(PIC_1_OFFSET) });

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

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
	instruction_pointer: u32,
	code_segment: u32,
	cpu_flags: u32,
	stack_pointer: u32,
	stack_segment: u32,
}

#[macro_export]
macro_rules! handler {
	($name: ident) => {{
		#[naked]
		extern "C" fn wrapper() {
			unsafe {
				asm!(
					// Set up stack frame
					"push ebp",
					"mov ebp, esp",

					// Save all general-purpose registers
					"pushad",

					// Calculate the correct stack frame pointer
					"mov eax, esp",
					"add eax, 36", // Adjust for 'pushad' and possibly other pushed registers
					"push eax", // Push stack frame pointer

					// Call the actual interrupt handler
					"call {}",

					// Restore all general-purpose registers
					"pop eax", // Clean up the stack
					"popad",

					// Restore base pointer and return from interrupt
					"pop ebp",
					"iretd",
					sym $name,
					options(noreturn)
				);
			}
		}
		wrapper as extern "C" fn()
	}};
}


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
	println!("EXCEPTION: BREAKPOINT at {:#x}\n{:#x?}", stack_frame.instruction_pointer, stack_frame);
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
	println!("EXCEPTION: COPROCESSOR SEGMENT OVERRUN\n{:#x?}", _stack_frame);
}

pub fn invalid_task_state_segment(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: INVALID TASK STATE SEGMENT\n{:#x?}", _stack_frame);
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
	println!("EXCEPTION: SIMD FLOATING POINT EXCEPTION\n{:#x?}", _stack_frame);
}

pub fn virtualization_exception(_stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: VIRTUALIZATION EXCEPTION\n{:#x?}", _stack_frame);
}

pub fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
}

pub fn keyboard_interrupt(_stack_frame: &mut InterruptStackFrame) {
	use crate::keyboard::{KEYBOARD_INTERRUPT_RECEIVED, LAST_SCANCODE};
	use core::sync::atomic::Ordering;
	let scancode: u8 = unsafe { inb(0x60) };

	*LAST_SCANCODE.lock() = scancode;
	KEYBOARD_INTERRUPT_RECEIVED.store(true, Ordering::SeqCst);

	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
	}
}

pub fn init() {
	unsafe {
		PICS.lock().initialize();
	}
	enable();
}

pub fn enable() {
	use core::arch::asm;
	unsafe {
		asm!("sti", options(preserves_flags, nostack));
	}
}

pub fn disable() {
	use core::arch::asm;
	unsafe {
		asm!("cli", options(preserves_flags, nostack));
	}
}
