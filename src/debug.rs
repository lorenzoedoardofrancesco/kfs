use crate::io::{ outb, inb };

const SERIAL_PORT: u16 = 0x3f8;

pub fn init_serial_port() {
	unsafe {
		outb(SERIAL_PORT + 1, 0x00);
		outb(SERIAL_PORT + 3, 0x80);
		outb(SERIAL_PORT + 0, 0x03);
		outb(SERIAL_PORT + 1, 0x00);
		outb(SERIAL_PORT + 3, 0x03);
		outb(SERIAL_PORT + 2, 0xc7);
		outb(SERIAL_PORT + 4, 0x0b);
	}
}

fn is_transmit_empty() -> bool {
	unsafe { (inb(SERIAL_PORT + 5) & 0x20) != 0 }
}

fn serial_write_byte(byte: u8) {
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
