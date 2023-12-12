use core::arch::asm;
use lazy_static::lazy_static;
use crate::interrupts::{ InterruptIndex, timer_interrupt, keyboard_interrupt, serial_println };

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IDT_Descriptor {
	offset_low: u16,
	selector: u16,
	zero: u8,
	type_attributes: u8,
	offset_high: u16,
}

impl IDT_Descriptor {
	fn new(offset: u32, selector: u16, type_attributes: u8) -> IDT_Descriptor {
		IDT_Descriptor {
			offset_low: (offset & 0xffff) as u16,
			selector: selector,
			zero: 0,
			type_attributes,
			offset_high: ((offset >> 16) & 0xffff) as u16,
		}
	}
}

extern "C" fn divide_by_zero() {
	serial_println("Divide by zero");
}

extern "C" fn debug() {
	serial_println("Debug");
}

extern "C" fn non_maskable_interrupt() {
	serial_println("Non-maskable interrupt");
}

extern "C" fn breakpoint() {
	serial_println("Breakpoint");
}

extern "C" fn overflow() {
	serial_println("Overflow");
}

extern "C" fn bound_range_exceeded() {
	serial_println("Bound range exceeded");
}

extern "C" fn invalid_opcode() {
	serial_println("Invalid opcode");
}

extern "C" fn coprocessor_not_available() {
	serial_println("Coprocessor not available");
}

extern "C" fn double_fault() {
	serial_println("Double fault");
}

extern "C" fn coprocessor_segment_overrun() {
	serial_println("Coprocessor segment overrun");
}

extern "C" fn invalid_task_state_segment() {
	serial_println("Invalid task state segment");
}

extern "C" fn segment_not_present() {
	serial_println("Segment not present");
}

extern "C" fn stack_fault() {
	serial_println("Stack fault");
}

extern "C" fn general_protection_fault() {
	serial_println("General protection fault");
}

extern "C" fn page_fault() {
	serial_println("Page fault");
}

extern "C" fn reserved() {
	serial_println("Reserved");
}

extern "C" fn math_fault() {
	serial_println("Math fault");
}

extern "C" fn alignment_check() {
	serial_println("Alignment check");
}

extern "C" fn machine_check() {
	serial_println("Machine check");
}

extern "C" fn simd_floating_point_exception() {
	serial_println("SIMD floating point exception");
}

extern "C" fn virtualization_exception() {
	serial_println("Virtualization exception");
}

lazy_static! {
	#[link_section = ".idt"]
	static ref IDT: [IDT_Descriptor; 256] = {
		let mut idt = [IDT_Descriptor::new(0, 0, 0); 256];

		idt[0] = IDT_Descriptor::new(divide_by_zero as u32, 0x08, 0x8e);
		idt[1] = IDT_Descriptor::new(debug as u32, 0x08, 0x8e);
		idt[2] = IDT_Descriptor::new(non_maskable_interrupt as u32, 0x08, 0x8e);
		idt[3] = IDT_Descriptor::new(breakpoint as u32, 0x08, 0x8e);
		idt[4] = IDT_Descriptor::new(overflow as u32, 0x08, 0x8e);
		idt[5] = IDT_Descriptor::new(bound_range_exceeded as u32, 0x08, 0x8e);
		idt[6] = IDT_Descriptor::new(invalid_opcode as u32, 0x08, 0x8e);
		idt[7] = IDT_Descriptor::new(coprocessor_not_available as u32, 0x08, 0x8e);
		idt[8] = IDT_Descriptor::new(double_fault as u32, 0x08, 0x8f);
		idt[9] = IDT_Descriptor::new(coprocessor_segment_overrun as u32, 0x08, 0x8f);
		idt[10] = IDT_Descriptor::new(invalid_task_state_segment as u32, 0x08, 0x8f);
		idt[11] = IDT_Descriptor::new(segment_not_present as u32, 0x08, 0x8f);
		idt[12] = IDT_Descriptor::new(stack_fault as u32, 0x08, 0x8f);
		idt[13] = IDT_Descriptor::new(general_protection_fault as u32, 0x08, 0x8f);
		idt[14] = IDT_Descriptor::new(page_fault as u32, 0x08, 0x8f);
		idt[15] = IDT_Descriptor::new(reserved as u32, 0x08, 0x8f);
		idt[16] = IDT_Descriptor::new(math_fault as u32, 0x08, 0x8e);
		idt[17] = IDT_Descriptor::new(alignment_check as u32, 0x08, 0x8f);
		idt[18] = IDT_Descriptor::new(machine_check as u32, 0x08, 0x8f);
		idt[19] = IDT_Descriptor::new(simd_floating_point_exception as u32, 0x08, 0x8e);
		idt[20] = IDT_Descriptor::new(virtualization_exception as u32, 0x08, 0x8f);
		idt[InterruptIndex::Timer.as_usize()] = IDT_Descriptor::new(
			timer_interrupt as u32,
			0x08,
			0x8e
		);
		idt[InterruptIndex::Keyboard.as_usize()] = IDT_Descriptor::new(
			keyboard_interrupt as u32,
			0x08,
			0x8e
		);
		idt
	};
}

#[repr(C, packed)]
struct IDT_Register {
	size: u16,
	offset: u32,
}

pub fn idt_init() {
	unsafe {
		let idt_register = IDT_Register {
			size: (core::mem::size_of::<[IDT_Descriptor; 256]>() - 1) as u16,
			offset: IDT.as_ptr() as u32,
		};

		asm!("lidt [{}]", in(reg) &idt_register, options(readonly, nostack, preserves_flags));
	}
}
