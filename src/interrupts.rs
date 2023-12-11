use core::arch::asm;
use crate::io::inb;
use crate::pic8259::ChainedPics;
use spin::Mutex;
use crate::video_graphics_array::WRITER;


pub const PIC_1_OFFSET: u8 = 20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

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
	pub fn as_u8(self) -> u8 {
		self as u8
	}

	pub fn as_usize(self) -> usize {
		usize::from(self.as_u8())
	}
}

pub extern "C" fn timer_interrupt() {
	WRITER.lock().write_byte(b'.');
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
}

pub extern "C" fn keyboard_interrupt() {
	let scancode: u8 = unsafe { inb(0x60) };
	WRITER.lock().write_byte(b'X');
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
	}
}

pub fn pics_init() {
	unsafe {
		PICS.lock().initialize();
	}
	enable();
}

pub fn enable() {
	unsafe {
		asm!("sti", options(preserves_flags, nostack));
	}
}
