//! # Shell Command Processing Module
//!
//! This module provides the core functionalities for a simple shell-like command-line interface.
//! It includes the implementation of basic shell commands, a readline function for processing
//! input lines, and specialized command handlers. The module allows interaction with the system
//! through a set of predefined commands.

use crate::exceptions::interrupts;
use crate::shell::history::HISTORY;
use crate::shell::prints::{help, print_stack, print_unknown_command};
use crate::utils::io::{outb, outw};
use crate::utils::librs::hlt;
use crate::utils::librs::{get_rtc_date, get_rtc_time};
use crate::vga::video_graphics_array::WRITER;

pub const MAX_LINE_LENGTH: usize = 76;
pub const MAX_HISTORY_LINES: usize = 16;

/// Clears the VGA text screen.
///
/// This function uses the VGA text buffer writer to clear the screen.
pub fn clear() {
	interrupts::disable();
	WRITER.lock().clear_screen();
	interrupts::enable();
}

/// Echoes the provided message to the VGA text screen.
///
/// This function prints the provided message back to the screen.
/// It is a basic implementation of the `echo` shell command.
fn echo(line: &str) {
	let message: &str = &line["echo".len()..];
	if message.starts_with(" ") && message.len() > 1 {
		println!("{}", message[1..].trim());
	} else {
		println!("echo: missing argument");
	}
}

/// Displays the current time.
///
/// Retrieves and prints the current time using the Real Time Clock.
fn time() {
	let (hours, minutes, seconds) = get_rtc_time();
	println!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}

/// Displays the current date.
///
/// Retrieves and prints the current date and time using the Real Time Clock.
fn date() {
	let (hours, minutes, seconds) = get_rtc_time();
	let (year, month, day) = get_rtc_date();

	let full_year = 2000 + year as u16;

	println!(
		"{:02}/{:02}/{:02} {:02}:{:02}:{:02}",
		day, month, full_year, hours, minutes, seconds
	);
}

/// Displays an amazing ASCII cat.
///
/// This function is an easter egg, printing a small ASCII art representation of a cat. Meow!
fn miao() {
	println!("  /\\_/\\");
	println!("=( ^.^ )=");
	println!("  )   (   //");
	println!(" (__ __)//");
}

/// Initiates a system reboot.
///
/// This function sends a command to the keyboard controller to trigger a CPU reset.
fn reboot() {
	unsafe {
		outb(0x64, 0xfe);
	}
}

/// Initiates system shutdown.
///
/// This function sends a command to initiate a system shutdown procedure.
fn shutdown() {
	unsafe {
		outw(0x604, 0x2000);
	}
}

/// Prints system information.
///
/// Displays information about the operating system, such as version, build date, and system name.
fn uname() {
	println!(
		"{} {} {} {} {} {}",
		"KFS",
		"0.4.2-kfs-i386",
		"Keystroke-Fusion-Surgery (2023-12-22)",
		"i386",
		"KFS/Deepnux",
		"A|L"
	);
}

/// Processes a line of input as a shell command.
///
/// This function takes a raw input line, trims it, and executes the corresponding shell command
/// if it matches a known command.
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

/// Handles commands that require additional parsing.
///
/// This function is called for commands that require additional parsing or are not part of the
/// standard command set, like `echo` and `stack`.
fn handle_special_commands(line: &str) {
	if line.starts_with("echo") {
		echo(line);
	} else if line.starts_with("stack") {
		print_stack(line);
	} else {
		print_unknown_command(line);
	}
}

