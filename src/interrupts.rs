use core::arch::asm;
use crate::io::inb;
use crate::pic8259::ChainedPics;
use spin::Mutex;
use crate::video_graphics_array::WRITER;

pub const PIC_1_OFFSET: u8 = 32;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
	ChainedPics::new_contiguous(PIC_1_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
	Timer = PIC_1_OFFSET,
	Keyboard,
}

impl InterruptIndex {
	pub fn as_u8(self) -> u8 {
		self as u8
	}

	pub fn as_usize(self) -> usize {
		usize::from(self.as_u8())
	}
}

#[allow(dead_code)]
pub struct InterruptStackFrame {
	 instruction_pointer: u32,
	code_segment: u32,
	cpu_flags: u32,
	stack_pointer: u32,
	stack_segment: u32,
}

pub extern "x86-interrupt" fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
	WRITER.lock().write_byte(b'.');
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
}

pub extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: &mut InterruptStackFrame) {
	let scancode = unsafe { inb(0x60) };
	WRITER.lock().write_byte(scancode);
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
	}
}

pub fn pics_init() {
	unsafe {
		PICS.lock().initialize();
	}
}

pub fn enable() {
	unsafe {
		asm!("sti", options(preserves_flags, nostack));
	}
}
