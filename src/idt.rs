use core::arch::asm;
use lazy_static::lazy_static;
use crate::interrupts::InterruptIndex;
use crate::interrupts::{ divide_by_zero, debug, non_maskable_interrupt, breakpoint, overflow, bound_range_exceeded, invalid_opcode, coprocessor_not_available, double_fault, coprocessor_segment_overrun, invalid_task_state_segment, segment_not_present, stack_fault, general_protection_fault, page_fault, reserved, math_fault, alignment_check, machine_check, simd_floating_point_exception, virtualization_exception, timer_interrupt, keyboard_interrupt };

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

static DIVIDE_BY_ZERO: extern "C" fn() = handler!(divide_by_zero);
static DEBUGG: extern "C" fn() = handler!(debug);
static NON_MASKABLE_INTERRUPT: extern "C" fn() = handler!(non_maskable_interrupt);
static BREAKPOINT: extern "C" fn() = handler!(breakpoint);
static OVERFLOW: extern "C" fn() = handler!(overflow);
static BOUND_RANGE_EXCEEDED: extern "C" fn() = handler!(bound_range_exceeded);
static INVALID_OPCODE: extern "C" fn() = handler!(invalid_opcode);
static COPROCESSOR_NOT_AVAILABLE: extern "C" fn() = handler!(coprocessor_not_available);
static DOUBLE_FAULT: extern "C" fn() = handler!(double_fault);
static COPROCESSOR_SEGMENT_OVERRUN: extern "C" fn() = handler!(coprocessor_segment_overrun);
static INVALID_TASK_STATE_SEGMENT: extern "C" fn() = handler!(invalid_task_state_segment);
static SEGMENT_NOT_PRESENT: extern "C" fn() = handler!(segment_not_present);
static STACK_FAULT: extern "C" fn() = handler!(stack_fault);
static GENERAL_PROTECTION_FAULT: extern "C" fn() = handler!(general_protection_fault);
static PAGE_FAULT: extern "C" fn() = handler!(page_fault);
static RESERVED: extern "C" fn() = handler!(reserved);
static MATH_FAULT: extern "C" fn() = handler!(math_fault);
static ALIGNMENT_CHECK: extern "C" fn() = handler!(alignment_check);
static MACHINE_CHECK: extern "C" fn() = handler!(machine_check);
static SIMD_FLOATING_POINT_EXCEPTION: extern "C" fn() = handler!(simd_floating_point_exception);
static VIRTUALIZATION_EXCEPTION: extern "C" fn() = handler!(virtualization_exception);
static TIMER_INTERRUPT: extern "C" fn() = handler!(timer_interrupt);
static KEYBOARD_INTERRUPT: extern "C" fn() = handler!(keyboard_interrupt);

lazy_static! {
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
		/*
		idt[InterruptIndex::Rtc.as_usize()] = IdtDescriptor::new(
			rtc_interrupt as u32,
			0x08,
			0x8e
		);
		 */
		idt
	};
}

#[repr(C, packed)]
struct IdtRegister {
	size: u16,
	offset: u32,
}

pub fn init() {
	unsafe {
		let idt_register = IdtRegister {
			size: (core::mem::size_of::<[IdtDescriptor; 256]>() - 1) as u16,
			offset: IDT.as_ptr() as u32,
		};

		asm!("lidt [{}]", in(reg) &idt_register, options(readonly, nostack, preserves_flags));
	}
}
