use crate::shell::history::HISTORY;
use crate::shell::prints::{help, print_stack, print_unknown_command};
use crate::utils::io::{outb, outw};
use crate::utils::librs::hlt;
use crate::utils::librs::{get_rtc_date, get_rtc_time};
use crate::vga::video_graphics_array::WRITER;

pub const MAX_LINE_LENGTH: usize = 76;
pub const MAX_HISTORY_LINES: usize = 16;

pub fn clear() {
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

pub fn readline(raw_line: &str) {
	let line = raw_line.trim();
	if line.is_empty() {
		return;
	}
	HISTORY.lock().add(raw_line);

	match line {
		"help" | "man" => help(),
		"clear" => clear(),
		"time" => time(),
		"miao" => miao(),
		"reboot" => reboot(),
		"halt" => hlt(),
		"shutdown" => shutdown(),
		"history" => HISTORY.lock().print(),
		"date" => date(),
		"uname" => uname(),
		_ => handle_special_commands(line),
	}
}

fn handle_special_commands(line: &str) {
	if line.starts_with("echo") {
		echo(line);
	} else if line.starts_with("stack") {
		print_stack(line);
	} else {
		print_unknown_command(line);
	}
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
