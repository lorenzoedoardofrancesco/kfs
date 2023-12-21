use crate::utils::io::{inb, outb};

use super::interrupts;

const RTC_COMMAND_REGISTER: u16 = 0x70;
const RTC_DATA_REGISTER: u16 = 0x71;

const RTC_REGISTER_B: u8 = 0x0B;
pub const RTC_REGISTER_C: u8 = 0x0C;

pub unsafe fn rtc_read_register(register: u8) -> u8 {
	outb(RTC_COMMAND_REGISTER, register);
	inb(RTC_DATA_REGISTER)
}

unsafe fn rtc_write_register(register: u8, value: u8) {
	outb(RTC_COMMAND_REGISTER, register);
	outb(RTC_DATA_REGISTER, value);
}

pub fn enable_rtc_interrupts() {
	interrupts::disable();
	unsafe {
		let register_b = rtc_read_register(RTC_REGISTER_B);
		rtc_write_register(RTC_REGISTER_B, register_b | 0x40);
	}
	interrupts::enable();
}
