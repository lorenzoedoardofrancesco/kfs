use spin::Mutex;
use crate::io::inb;
use crate::pic8259::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
	ChainedPics::new_contiguous(PIC_1_OFFSET)
});

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

#[allow(dead_code)]
pub struct InterruptStackFrame {
	instruction_pointer: u32,
	code_segment: u32,
	cpu_flags: u32,
	stack_pointer: u32,
	stack_segment: u32,
}

pub extern "x86-interrupt" fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
}

pub extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: &mut InterruptStackFrame) {
	use core::sync::atomic::{ Ordering };
	use crate::keyboard::{ KEYBOARD_INTERRUPT_RECEIVED, LAST_SCANCODE };
	let scancode: u8 = unsafe { inb(0x60) };

	*LAST_SCANCODE.lock() = scancode;
	KEYBOARD_INTERRUPT_RECEIVED.store(true, Ordering::SeqCst);

	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
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
