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

// IDT     Interrupt Descriptor Table

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct IdtDescriptor {
	offset_low: u16,
	selector: u16,
	zero: u8,
	type_attributes: u8,
	offset_high: u16,
}

impl IdtDescriptor {
    fn new(offset: u32, selector: u16, type_attributes: u8) -> IdtDescriptor {
        IdtDescriptor {
            offset_low: (offset & 0xffff) as u16,
            selector: selector,     // see in the GDT which code segment is used
            zero: 0,
            type_attributes,
            offset_high: ((offset >> 16) & 0xffff) as u16,
        }
    }
}

#[link_section = ".idt"]
lazy_static! {
    static ref DEFAULT_IDT_DESCRIPTOR: IdtDescriptor = IdtDescriptor::new(0, 0, 0);
    
    static ref IDT: [IdtDescriptor; 255] = {
        let mut idt_entries = [DEFAULT_IDT_DESCRIPTOR; 255];
        
        // Override specific entries if needed
        idt_entries[0] = IdtDescriptor::new(0x08, 0, 0x8e)
        idt_entries[1] = IdtDescriptor::new(0x08, 0, 0x8e)
        
        idt_entries
    };
}

// #[derive(Debug, Clone, Copy)]
// #[repr(C)]
// struct InterruptDescriptorTable {
//     descriptors: [IdtDescriptor; 256],
// }

// impl InterruptDescriptorTable {
//     fn new() -> InterruptDescriptorTable {
//         InterruptDescriptorTable {
//             descriptors[0..256]: [IdtDescriptor::new(0, 0, 0); 256],
//         }
//     }


//     fn add_entry(&mut self, index: usize, offset: u32, selector: u16, type_attributes: u8) {
//         self.descriptors[index] = IdtDescriptor::new(offset, selector, type_attributes);
//     }   
// }
