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
					// Save scratch registers
					"push eax",
					"push ecx",
					"push edx",
					"push ebx",

					// Setup for interrupt handler call
					"mov eax, esp",
					"add eax, 4*4",
					"push eax",
					"call {}",

					// Restore scratch registers
					"pop ebx",
					"pop edx",
					"pop ecx",
					"pop eax",

					// Return from interrupt
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
	println!("EXCEPTION: DIVIDE BY ZERO\n");
}

pub extern "C" fn debug(_stack_frame: &mut InterruptStackFrame) {
	println!("Debug");
}

pub extern "C" fn non_maskable_interrupt(_stack_frame: &mut InterruptStackFrame) {
	println!("Non-maskable interrupt");
}

pub extern "C" fn breakpoint(_stack_frame: &mut InterruptStackFrame) {
	print!("instruction_pointer: {:#08x}, code_segment: {:#08x}, cpu_flags: {:#08x}, stack_pointer: {:#08x}, stack_segment: {:#08x}\n", _stack_frame.instruction_pointer, _stack_frame.code_segment, _stack_frame.cpu_flags, _stack_frame.stack_pointer, _stack_frame.stack_segment);
	crate::librs::print_stack();
}

pub fn overflow(_stack_frame: &mut InterruptStackFrame) {
	println!("Overflow");
}

pub fn bound_range_exceeded(_stack_frame: &mut InterruptStackFrame) {
	println!("Bound range exceeded");
}

pub fn invalid_opcode(_stack_frame: &mut InterruptStackFrame) {
	println!("Invalid opcode");
}

pub fn coprocessor_not_available(_stack_frame: &mut InterruptStackFrame) {
	println!("Coprocessor not available");
}

pub fn double_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("Double fault");
}

pub fn coprocessor_segment_overrun(_stack_frame: &mut InterruptStackFrame) {
	println!("Coprocessor segment overrun");
}

pub fn invalid_task_state_segment(_stack_frame: &mut InterruptStackFrame) {
	println!("Invalid task state segment");
}

pub fn segment_not_present(_stack_frame: &mut InterruptStackFrame) {
	println!("Segment not present");
}

pub fn stack_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("Stack fault");
}

pub fn general_protection_fault(_stack_frame: &mut InterruptStackFrame) {
	print_serial!("instruction_pointer: {:#08x}, code_segment: {:#08x}, cpu_flags: {:#08x}, stack_pointer: {:#08x}, stack_segment: {:#08x}\n", _stack_frame.instruction_pointer, _stack_frame.code_segment, _stack_frame.cpu_flags, _stack_frame.stack_pointer, _stack_frame.stack_segment);
}

pub fn page_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("Page fault");
}

pub fn reserved(_stack_frame: &mut InterruptStackFrame) {
	println!("Reserved");
}

pub fn math_fault(_stack_frame: &mut InterruptStackFrame) {
	println!("Math fault");
}

pub fn alignment_check(_stack_frame: &mut InterruptStackFrame) {
	println!("Alignment check");
}

pub fn machine_check(_stack_frame: &mut InterruptStackFrame) {
	println!("Machine check");
}

pub fn simd_floating_point_exception(_stack_frame: &mut InterruptStackFrame) {
	println!("SIMD floating point exception");
}

pub fn virtualization_exception(_stack_frame: &mut InterruptStackFrame) {
	println!("Virtualization exception");
}

pub extern "x86-interrupt" fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
	unsafe {
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
}

pub extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: &mut InterruptStackFrame) {
	use crate::keyboard::{KEYBOARD_INTERRUPT_RECEIVED, LAST_SCANCODE};
	use core::sync::atomic::Ordering;
	let scancode: u8 = unsafe { inb(0x60) };

	*LAST_SCANCODE.lock() = scancode;
	KEYBOARD_INTERRUPT_RECEIVED.store(true, Ordering::SeqCst);

	unsafe {
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
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
