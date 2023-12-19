use crate::generate_interrupt;
use crate::librs::{self, hexdump, printraw};
use crate::utils::io::{inb, outb, outw};
use crate::vga::{prompt::PROMPT, video_graphics_array::WRITER};
use lazy_static::lazy_static;
use spin::Mutex;

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

const MAX_LINE_LENGTH: usize = 76;
const MAX_HISTORY_LINES: usize = 16;

pub struct History {
	buffer: [[u8; MAX_LINE_LENGTH]; MAX_HISTORY_LINES],
	last_input: [u8; MAX_LINE_LENGTH],
	index: usize,
	add_index: usize,
}

fn u8_array_cmp(a: &[u8; MAX_LINE_LENGTH], b: &[u8; MAX_LINE_LENGTH]) -> bool {
	for i in 0..MAX_LINE_LENGTH {
		if a[i] != b[i] {
			return false;
		}
	}
	true
}

impl History {
	fn new() -> History {
		History {
			buffer: [[0; MAX_LINE_LENGTH]; MAX_HISTORY_LINES],
			last_input: [0; MAX_LINE_LENGTH],
			index: 0,
			add_index: 0,
		}
	}

	fn add(&mut self, line: &str) {
		let line_u8 = str_to_array(line);

		if u8_array_cmp(&line_u8, &self.last_input) {
			self.index = self.add_index;
			return;
		}
		self.buffer[self.add_index] = line_u8;
		self.last_input = line_u8;
		self.add_index = (self.add_index + 1) % MAX_HISTORY_LINES;
		self.index = self.add_index;

	}

	fn get(&self, index: usize) -> &[u8; MAX_LINE_LENGTH] {
		&self.buffer[index]
	}

	fn print(&self) {
		for i in 0..MAX_HISTORY_LINES {
			let line = self.get(i);
			if line[0] != 0 {
				for &c in line.iter().take_while(|&&c| c != 0) {
					print!("{}", c as char);
				}
				println!();
			}
		}
	}

	fn print_prompt(&self, index: usize) {
		for c in self.get(index).iter().take_while(|&&c| c != 0) {
			PROMPT.lock().insert_char(*c, false);
		}
	}

	pub fn scroll_up(&mut self) {
		if self.index == 0 {
			if self.get(MAX_HISTORY_LINES - 1)[0] == 0 {
				return;
			}
			self.index = MAX_HISTORY_LINES - 1;
		} else {
			self.index = (self.index - 1) % MAX_HISTORY_LINES;
		}

		PROMPT.lock().init();
		self.print_prompt(self.index);
	}

	pub fn scroll_down(&mut self) {
		if self.index == MAX_HISTORY_LINES - 1 {
			self.index = 0;
		} else {
			if self.get(self.index + 1)[0] == 0 {
				return;
			}
			self.index = (self.index + 1) % MAX_HISTORY_LINES;
		}

		PROMPT.lock().init();
		self.print_prompt(self.index);
	}
}

lazy_static! {
	pub static ref HISTORY: Mutex<History> = Mutex::new(History::new());
}

fn str_to_array(s: &str) -> [u8; MAX_LINE_LENGTH] {
	let mut array = [0; MAX_LINE_LENGTH];
	for (i, c) in s.bytes().enumerate() {
		array[i] = c;
	}
	array
}

fn bcd_to_binary(bcd: u8) -> u8 {
	((bcd & 0xf0) >> 4) * 10 + (bcd & 0x0f)
}

fn print_help_line(command: &str, description: &str) {
	print!("  {:13}", command);
	printraw("Z");
	print!("  {:60}", description);
	if command == "shutdown" {
		printraw("Z");
	} else if command != "F12" {
		printraw("ZZ");
	}
}

fn help() {
	clear();
	printraw("immmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm[Z");
	print!(" Available commands                                                           ");
	printraw("ZlmmmmmmmmmmmmmmmkmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmYZ");
	print_help_line("echo", "display a line of text");
	print_help_line("clear", "clear the screen");
	print_help_line("printstack", "print the stack");
	print_help_line("time", "print the time");
	print_help_line("date", "display the current date and time");
	print_help_line("miao", "print a cat");
	print_help_line("uname", "print system information");
	print_help_line("except", "throw an exception");
	print_help_line("halt", "halt the system");
	print_help_line("reboot", "reboot the system");
	print_help_line("shutdown", "shutdown the system");
	printraw("lmmmmmmmmmmmmmmmnmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmYZ");
	print_help_line("F1-F4", "change between screens");
	print_help_line("F9", "display welcome message");
	print_help_line("F10", "change keyboard layout");
	print_help_line("F11", "switch text color");
	print_help_line("F12", "switch background color");

	printraw("ZlmmmmmmmmmmmmmmmjmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmYZ");
	print!(
		" Type 'history' to view command history           {} {} navigate history        ",
		0x1e as char, 0x1f as char
	);
	printraw("Zhmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm\\");
	println!("");
}

fn clear() {
	WRITER.lock().clear_screen();
}

fn echo(line: &str) {
	let message: &str = &line["echo".len()..];
	if message.starts_with(" ") && message.len() > 1 {
		println!("{}", message[1..].trim());
	} else {
		println!("echo: missing argument");
	}
}

fn read_cmos(register: u8) -> u8 {
	unsafe {
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

fn get_rtc_date() -> (u8, u8, u8) {
	let year = bcd_to_binary(read_cmos(0x09));
	let month = bcd_to_binary(read_cmos(0x08));
	let day = bcd_to_binary(read_cmos(0x07));

	(year, month, day)
}

fn time() {
	let (hours, minutes, seconds) = get_rtc_time();
	println!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}

fn date() {
	let (hours, minutes, seconds) = get_rtc_time();
	let (year, month, day) = get_rtc_date();

	let full_year = 2000 + year as u16;

	println!(
		"{:02}/{:02}/{:02} {:02}:{:02}:{:02}",
		day, month, full_year, hours, minutes, seconds
	);
}

fn miao() {
	println!("  /\\_/\\");
	println!("=( ^.^ )=");
	println!("  )   (   //");
	println!(" (__ __)//");
}

fn reboot() {
	unsafe {
		outb(0x64, 0xfe);
	}
}

fn shutdown() {
	unsafe {
		outw(0x604, 0x2000);
	}
}

fn uname() {
	println!(
		"{} {} {} {} {} {}",
		"KFC",
		"0.0.1-kfc1-i386",
		"DeepFryer 0.0.1-1kfc1 (2023-12-13)",
		"i386",
		"KFC/Deepnux",
		"A|L"
	);
}

fn except(line: &str) {
	let message: &str = &line["except".len()..];
	if message.starts_with(" ") && message.len() > 1 {
		let num: usize = message[1..].trim().parse::<usize>().unwrap_or(usize::MAX);
		if num > 255 {
			println!("except: argument must be between 0 and 255");
			return;
		}
		println!("except: throwing exception {}", num);
		generate_interrupt(num as u8);
	} else {
		println!("except: missing argument");
	}
}

use crate::{ESP, EBP};

pub fn readline(raw_line: &str) {
	let line = raw_line.trim();
	if line.is_empty() {
		return;
	}
	HISTORY.lock().add(raw_line);
	match line {
		"help" | "man" => help(),
		"clear" => clear(),
		"stack" => unsafe { hexdump(ESP, (EBP - ESP) as usize) },
		"time" => time(),
		"miao" => miao(),
		"reboot" => reboot(),
		"halt" => librs::hlt(),
		"shutdown" => shutdown(),
		"history" => HISTORY.lock().print(),
		"date" => date(),
		"uname" => uname(),
		_ => {
			if line.starts_with("echo") {
				echo(line);
			} else if line.starts_with("except") {
				except(line);
			} else {
				let mut len = line.len();
				if len > 50 {
					len = 50;
				}
				println!("Unknown command: {}", line[0..len].trim());
			}
		}
	}
}

pub fn print_welcome_message() {
	librs::clear();
	println!("                                     :---------:    .---------:---------- ");
	println!("                                   :#@@@@@@@@%=     +@@@@@@@#::@@@@@@@@@@.");
	println!("                                 :#@@@@@@@@%=       +@@@@@%:  :@@@@@@@@@@.");
	println!("                               :#@@@@@@@@%=         +@@@%-    :@@@@@@@@@@.");
	println!("                             :#@@@@@@@@@=           +@%-      :@@@@@@@@@@.");
	println!("                           :#@@@@@@@@@=             =-        -@@@@@@@@@@ ");
	println!("                         :#@@@@@@@@@=                        +@@@@@@@@@*. ");
	println!("                       :#@@@@@@@@@=                        +@@@@@@@@@*.   ");
	println!("                     :#@@@@@@@@@=                        +@@@@@@@@@*.     ");
	println!("                   :#@@@@@@@@@=                        +@@@@@@@@@*.       ");
	println!("                 :#@@@@@@@@@=                        +@@@@@@@@@+.         ");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#        :#.");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#      :#@@.");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#    :#@@@@.");
	println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#  :#@@@@@@.");
	println!("                 ....................=@@@@@@@@@@    +@@@@@@@@@#:#@@@@@@@@.");
	println!("                                     -@@@@@@@@@@     .................... ");
	println!("                                     -@@@@@@@@@@     by                   ");
	println!("                                     -@@@@@@@@@@          Alix Muller     ");
	println!("                                     -@@@@@@@@@@       Lorenzo Simanic    ");
	println!("                                     .----------                          ");
	println!("");
	println!("                       Welcome to KFC! Type 'help' for a list of commands!");
	PROMPT.lock().init();
}

// Y    ┤
// Z    ||
// [    ┐
// \\   ┘
// h    └
// i    ┌
// j    ┴
// k    ┬
// l    ├
// m    ─
// n    ┼
