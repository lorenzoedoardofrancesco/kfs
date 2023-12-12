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

pub extern "C" fn timer_interrupt() {
	serial_println("Timer interrupt");
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	}
}

pub extern "C" fn keyboard_interrupt() {
	let scancode: u8 = unsafe { inb(0x60) };
	WRITER.lock().write_byte(scancode);
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
	}
}

pub fn pics_init() {
	init_serial_port();
	unsafe {
		PICS.lock().initialize();
		PICS.lock().write_masks(0b11111101, 0b11111111);
	}
	enable();
}

pub fn enable() {
	unsafe {
		asm!("sti", options(preserves_flags, nostack));
	}
}

// DEBUG

const SERIAL_PORT: u16 = 0x3f8; // COM1

fn init_serial_port() {
	use crate::io::{ outb, inb };

	unsafe {
		outb(SERIAL_PORT + 1, 0x00); // Disable all interrupts
		outb(SERIAL_PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
		outb(SERIAL_PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
		outb(SERIAL_PORT + 1, 0x00); //                  (hi byte)
		outb(SERIAL_PORT + 3, 0x03); // 8 bits, no parity, one stop bit
		outb(SERIAL_PORT + 2, 0xc7); // Enable FIFO, clear them, with 14-byte threshold
		outb(SERIAL_PORT + 4, 0x0b); // IRQs enabled, RTS/DSR set
	}
}

fn is_transmit_empty() -> bool {
	use crate::io::inb;

	unsafe { (inb(SERIAL_PORT + 5) & 0x20) != 0 }
}

fn serial_write_byte(byte: u8) {
	use crate::io::outb;

	while !is_transmit_empty() {}
	unsafe {
		outb(SERIAL_PORT, byte);
	}
}

pub fn serial_print(s: &str) {
	for byte in s.bytes() {
		serial_write_byte(byte);
	}
}

pub fn serial_println(s: &str) {
	serial_print(s);
	serial_print("\n\r");
}
