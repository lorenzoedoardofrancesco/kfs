use core::arch::asm;
use crate::pic8259::ChainedPics;
use spin::Mutex;

// IDT     Interrupt Descriptor Table

#[derive(Debug, Clone, Copy)]
struct IdtDescriptor {
	offset_low: u16,
	selector: u16,
	zero: u8,
	type_attributes: u8,
	offset_high: u16,
}




//	Hardware Interrupts 

pub const PIC_1_OFFSET: u8 = 20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub fn enable() {
    unsafe {
        asm!("sti", options(preserves_flags, nostack));
    }
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
