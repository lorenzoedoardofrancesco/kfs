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
        idt_entries[0] = 
        idt_entries[1] = IdtDescriptor::new(0xfffff, 0, 0x9a, 0xcf);
        idt_entries[2] = IdtDescriptor::new(0xfffff, 0, 0x92, 0xcf);
        
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
