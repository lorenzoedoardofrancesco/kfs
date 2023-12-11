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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct InterruptDescriptorTable {
    descriptors: [IdtDescriptor; 256],
}

impl InterruptDescriptorTable {
    fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            descriptors[0..256]: [IdtDescriptor::new(0, 0, 0); 256],
        }
    }
}