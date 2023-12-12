// Interrupt Vector Table
// Base Address	Interrupt Number	Description
// 0x000	0	Divide by 0
// 0x004	1	Single step (Debugger)
// 0x008	2	Non Maskable Interrupt (NMI) Pin
// 0x00C	3	Breakpoint (Debugger)
// 0x010	4	Overflow
// 0x014	5	Bounds check
// 0x018	6	Undefined Operation Code (OPCode) instruction
// 0x01C	7	No coprocessor
// 0x020	8	Double Fault
// 0x024	9	Coprocessor Segment Overrun
// 0x028	10	Invalid Task State Segment (TSS)
// 0x02C	11	Segment Not Present
// 0x030	12	Stack Segment Overrun
// 0x034	13	General Protection Fault (GPF)
// 0x038	14	Page Fault
// 0x03C	15	Unassigned
// 0x040	16	Coprocessor error
// 0x044	17	Alignment Check (486+ Only)
// 0x048	18	Machine Check (Pentium/586+ Only)
// 0x05C	19-31	Reserved exceptions
// 0x068 - 0x3FF	32-255	Interrupts free for software use


// IDTR Processor Register
// The IDTR register is the processor register that stores the base address of the IDT.

// The IDTR register has the following format:
// IDTR Register
// Bits 16...46 (IDT Base Address) 	Bits 0...15 (IDT Limit)

// x86 Processor Exceptions
// Interrupt Number	Class	Description	Error Code
// 0	Fault	Divide by 0	None
// 1	Trap or Fault	Single step (Debugger)	None. Can be retrived from debug registers
// 2	Unclassed	Non Maskable Interrupt (NMI) Pin	Not applicable
// 3	Trap	Breakpoint (Debugger)	None
// 4	Trap	Overflow	None
// 5	Fault	Bounds check	None
// 6	Fault	Unvalid OPCode	None
// 7	Fault	Device not available	None
// 8	Abort	Double Fault	Always 0
// 9	Abort (Reserved, do not use)	Coprocessor Segment Overrun	None
// 10	Fault	Invalid Task State Segment (TSS)	See error code below
// 11	Fault	Segment Not Present	See error code below
// 12	Fault	Stack Fault Exception	See error code below
// 13	Fault	General Protection Fault (GPF)	See error code below
// 14	Fault	Page Fault	See error code below
// 15	-	Unassigned	-
// 16	Fault	x87 FPU Error	None. x87 FPU provides own error information
// 17	Fault	Alignment Check (486+ Only)	Always 0
// 18	Abort	Machine Check (Pentium/586+ Only)	None. Error information abtained from MSRs
// 19	Fault	SIMD FPU Exception	None
// 20-31	-	Reserved	-
// 32-255	-	Avilable for software use	Not applicable

// x86 Hardware Interrupts
// 8259A Input pin	Interrupt Number	Description
// IRQ0	0x08	Timer
// IRQ1	0x09	Keyboard
// IRQ2	0x0A	Cascade for 8259A Slave controller
// IRQ3	0x0B	Serial port 2
// IRQ4	0x0C	Serial port 1
// IRQ5	0x0D	AT systems: Parallel Port 2. PS/2 systems: reserved
// IRQ6	0x0E	Diskette drive
// IRQ7	0x0F	Parallel Port 1
// IRQ8/IRQ0	0x70	CMOS Real time clock
// IRQ9/IRQ1	0x71	CGA vertical retrace
// IRQ10/IRQ2	0x72	Reserved
// IRQ11/IRQ3	0x73	Reserved
// IRQ12/IRQ4	0x74	AT systems: reserved. PS/2: auxiliary device
// IRQ13/IRQ5	0x75	FPU
// IRQ14/IRQ6	0x76	Hard disk controller
// IRQ15/IRQ7	0x77	Reserved

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
