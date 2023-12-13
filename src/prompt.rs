use lazy_static::lazy_static;
use spin::Mutex;
use crate::video_graphics_array::{ WRITER, VGA_COLUMNS, VGA_LAST_LINE };
//a delete vvvv
use crate::librs;

static PROMPT_STRING: &str = "$> ";
static PROMPT_LENGTH: usize = PROMPT_STRING.len();

lazy_static! {
	pub static ref PROMPT: Mutex<Prompt> = Mutex::new(Prompt {
		buffer: [0; VGA_COLUMNS],
		length: 0,
	});
}

pub struct Prompt {
	buffer: [u8; VGA_COLUMNS],
	pub length: usize,
}

impl Prompt {
	pub fn insert_string(&mut self, s: &str) {
		for c in s.bytes() {
			self.insert_char(c, false);
		}
	}

	pub fn insert_char(&mut self, c: u8, insert: bool) {
		if c == b'\n' {
			println!();

			// TIME A METTRE DE COTE

			const CMOS_ADDRESS: u16 = 0x70;
			const CMOS_DATA: u16 = 0x71;

			fn bcd_to_binary(bcd: u8) -> u8 {
				((bcd & 0xF0) >> 4) * 10 + (bcd & 0x0F)
			}

			fn read_cmos(register: u8) -> u8 {
				unsafe {
					use crate::io::{ inb, outb };
					outb(CMOS_ADDRESS, register);
					inb(CMOS_DATA)
				}
			}

			fn get_rtc_time() -> (u8, u8, u8) {
				let seconds = bcd_to_binary(read_cmos(0x00));
				let minutes = bcd_to_binary(read_cmos(0x02));
				let hours = bcd_to_binary(read_cmos(0x04));
			
				(hours, minutes, seconds)
			}
			//example juste pour TEST a refaire proprement sur shell.rs
			let buffer = core::str::from_utf8(&self.buffer[PROMPT_LENGTH..self.length]).unwrap();
			if buffer == "help" {
				println!("Available commands:");
				println!("    help   | display this help message");
				println!("    clear  | clear the screen");
				println!("    echo   | display the arguments");
				println!("    printk | print the stack");
				println!("    time   | print the time");
				println!("    miao   | print a cat");
			} else if buffer == "clear" {
				WRITER.lock().clear_screen();
			} else if buffer.starts_with("echo") {
				let message = &buffer["echo".len()..];
				if message.starts_with(" ") && message.len() > 1 {
					println!("{}", message[1..].trim());
				} else {
					println!("echo: missing argument");
				}
			} else if buffer == "printk" {
				librs::print_stack();
			} else if buffer == "time" {
				let (hours, minutes, seconds) = get_rtc_time();
				println!("{:02}:{:02}:{:02}", hours, minutes, seconds);
			} else if buffer == "miao" {
				println!("  /\\_/\\");
				println!("=( ^.^ )=");
				println!("  )   (   //");
				println!(" (__ __)//");
			} else if buffer != "" {
				println!("Unknown command: {}", buffer);
			}

			self.init();
			return;
		}

		if self.length == VGA_COLUMNS - 1 {
			return;
		}

		let column_position = WRITER.lock().column_position;
		if !insert {
			for i in (column_position..self.length).rev() {
				self.buffer[i + 1] = self.buffer[i];
			}
		}

		if !insert || column_position == self.length {
			self.length += 1;
		}

		self.buffer[column_position] = c;
		self.update_line();
		WRITER.lock().move_cursor(1);
	}

	pub fn remove_char(&mut self) {
		let column_position = WRITER.lock().column_position - 1;
		for i in column_position..self.length {
			self.buffer[i] = self.buffer[i + 1];
		}

		self.length -= 1;
		self.update_line();
		WRITER.lock().move_cursor(-1);
	}

	pub fn clear(&mut self) {
		for i in 0..self.length {
			self.buffer[i] = 0;
		}
		self.length = 0;
	}

	pub fn update_line(&mut self) {
		let buffer_as_str = core::str::from_utf8(&self.buffer[..self.length]).unwrap();
		WRITER.lock().update_line(buffer_as_str);
	}

	pub fn init(&mut self) {
		self.clear();
		self.insert_string(PROMPT_STRING);
	}
}

pub fn right_arrow() {
	if WRITER.lock().column_position < PROMPT.lock().length {
		WRITER.lock().move_cursor(1);
	}
}

pub fn left_arrow() {
	if WRITER.lock().column_position > PROMPT_LENGTH {
		WRITER.lock().move_cursor(-1);
	}
}

pub fn backspace() {
	if WRITER.lock().column_position > PROMPT_LENGTH {
		PROMPT.lock().remove_char();
	}
}

pub fn tab() {
	if WRITER.lock().column_position < VGA_COLUMNS - 4 {
		PROMPT.lock().insert_string("    ");
	}
}

pub fn end() {
	WRITER.lock().update_cursor(VGA_LAST_LINE, PROMPT.lock().length);
}

pub fn home() {
	WRITER.lock().update_cursor(VGA_LAST_LINE, PROMPT_LENGTH);
}

pub fn delete() {
	if WRITER.lock().column_position < PROMPT.lock().length {
		WRITER.lock().move_cursor(1);
		PROMPT.lock().remove_char();
	}
}
