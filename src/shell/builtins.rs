//! # Shell Command Processing Module
//!
//! This module provides the core functionalities for a simple shell-like command-line interface.
//! It includes the implementation of basic shell commands, a readline function for processing
//! input lines, and specialized command handlers. The module allows interaction with the system
//! through a set of predefined commands.

use crate::exceptions::interrupts::{self, TICKS};
use crate::shell::history::HISTORY;
use crate::shell::prints::PrintStackMode;
use crate::shell::prints::{help, print_stack, print_unknown_command};
use crate::utils::debug::LogLevel;
use crate::utils::io::{outb, outw};
use crate::utils::librs::hlt;
use crate::utils::librs::{get_rtc_date, get_rtc_time};
use crate::vga::video_graphics_array::WRITER;
use core::sync::atomic::Ordering;

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

/// Prints the current CPU mode.
///
/// Prints the current CPU mode (Real or Protected) along with the value of CR0.
fn cmd_mode() {
	let cr0: usize;
	unsafe {
		core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nostack, preserves_flags));
	}

	let mode = if cr0 & 1 == 0 { "real" } else { "protected" };
	log!(
		LogLevel::Info,
		"System is running in {} mode. CR0: {:b}",
		mode,
		cr0
	);
	println!("System is running in {} mode.", mode);
	describe_cr0(cr0);
}

/// Prints the value of CR0.
///
/// Prints the value of CR0 and the meaning of each flag.
fn describe_cr0(cr0: usize) {
	log!(LogLevel::Info, "CR0 Register: 0b{:032b}", cr0);
	println!("CR0 Register: 0b{:032b}", cr0);

	let flags = [
		("PE (Protection Enable)", cr0 & (1 << 0) != 0),
		("MP (Monitor Co-processor)", cr0 & (1 << 1) != 0),
		("EM (Emulation)", cr0 & (1 << 2) != 0),
		("TS (Task Switched)", cr0 & (1 << 3) != 0),
		("ET (Extension Type)", cr0 & (1 << 4) != 0),
		("NE (Numeric Error)", cr0 & (1 << 5) != 0),
		("WP (Write Protect)", cr0 & (1 << 16) != 0),
		("AM (Alignment Mask)", cr0 & (1 << 18) != 0),
		("NW (Not Write-through)", cr0 & (1 << 29) != 0),
		("CD (Cache Disable)", cr0 & (1 << 30) != 0),
		("PG (Paging)", cr0 & (1 << 31) != 0),
	];

	for (name, active) in flags.iter() {
		if *active {
			log!(LogLevel::Info, "{}: Activated", name);
			println!("{}: Activated", name);
		}
	}
}

/// Prints CPU information.
///
/// Prints the CPU vendor and brand information.
fn cpu_info() {
	let mut cpu_vendor = [0u8; 12];
	let mut cpu_brand = [0u8; 48];
	let mut eax: usize = 0;
	let mut ebx: usize = 0;
	let mut ecx: usize = 0;
	let mut edx: usize = 0;

	// Get CPU Vendor
	get_cpuid(0, &mut eax, &mut ebx, &mut ecx, &mut edx);
	cpu_vendor[0..4].copy_from_slice(&ebx.to_ne_bytes());
	cpu_vendor[4..8].copy_from_slice(&edx.to_ne_bytes());
	cpu_vendor[8..12].copy_from_slice(&ecx.to_ne_bytes());

	// Get CPU Brand
	for i in 0x80000002..=0x80000004 {
		get_cpuid(i, &mut eax, &mut ebx, &mut ecx, &mut edx);
		let offset = (i - 0x80000002) * 16;
		cpu_brand[offset..offset + 4].copy_from_slice(&eax.to_ne_bytes());
		cpu_brand[offset + 4..offset + 8].copy_from_slice(&ebx.to_ne_bytes());
		cpu_brand[offset + 8..offset + 12].copy_from_slice(&ecx.to_ne_bytes());
		cpu_brand[offset + 12..offset + 16].copy_from_slice(&edx.to_ne_bytes());
	}

	let cpu_vendor_str = core::str::from_utf8(&cpu_vendor).unwrap_or("Unknown");
	let cpu_brand_str = core::str::from_utf8(&cpu_brand).unwrap_or("Unknown");

	println!("CPU Vendor: {}", cpu_vendor_str);
	println!("CPU Brand: {}", cpu_brand_str);
}

/// Gets CPU information using the CPUID instruction.
///
/// This function uses the CPUID instruction to get information about the CPU.
fn get_cpuid(info_type: usize, eax: &mut usize, ebx: &mut usize, ecx: &mut usize, edx: &mut usize) {
	unsafe {
		core::arch::asm!(
			"cpuid",
			in("eax") info_type,
			lateout("eax") *eax,
			lateout("ebx") *ebx,
			lateout("ecx") *ecx,
			lateout("edx") *edx,
			options(nostack, nomem, preserves_flags)
		);
	}
}

/// Prints the system uptime.
///
/// This function prints the system uptime in the format `hh:mm:ss`.
fn show_uptime() {
	let uptime_seconds = TICKS.load(Ordering::SeqCst) / 18;

	let hours = (uptime_seconds % 86400) / 3600;
	let minutes = (uptime_seconds % 3600) / 60;
	let seconds = uptime_seconds % 60;

	println!(
		"System uptime: {:02}h:{:02}m:{:02}s",
		hours, minutes, seconds
	);
}

// Function to manually trigger a syscall for testing purposes
pub fn trigger_syscall(syscall_number: u32, arg1: u32, arg2: u32, arg3: u32) {
	use crate::exceptions::syscalls::GeneralRegs;
	let mut regs = GeneralRegs {
		eax: syscall_number, // Syscall number
		ebx: arg1,           // First argument
		ecx: arg2,           // Second argument
		edx: arg3,           // Third argument
		esi: 0,              // Additional registers if needed
		edi: 0,              // Additional registers if needed
		ebp: 0,              // Additional registers if needed
	};

	crate::exceptions::syscalls::syscall(&mut regs);
}

// Example shell command to trigger a specific syscall
// Example shell command to trigger a specific syscall
fn test_syscall(line: &str) {
	// Splitting the line by whitespace and extracting arguments
	let mut parts = [""; 5]; // syscall name, syscall number, arg1, arg2, arg3
	let mut part_index = 0;

	for word in line.split_whitespace() {
		parts[part_index] = word;
		part_index += 1;
		if part_index >= parts.len() {
			break;
		}
	}

	if part_index < 3 {
		println!("Usage: test_syscall <syscall_number> <arg1> <arg2> <arg3>");
		return;
	}

	let syscall_number = parts[1].parse::<u32>().unwrap_or(0);
	let arg1 = parts[2].parse::<u32>().unwrap_or(0);
	let arg2 = parts[3].parse::<u32>().unwrap_or(0);
	let arg3 = parts[4].parse::<u32>().unwrap_or(0);

	trigger_syscall(syscall_number, arg1, arg2, arg3);
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
		"uptime" => show_uptime(),
		"cpu" => cpu_info(),
		"mode" => cmd_mode(),
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
		print_stack(line, PrintStackMode::Vga);
	} else if line.starts_with("hexdump") {
		print_stack(line, PrintStackMode::Serial);
	} else if line.starts_with("test_syscall") {
		test_syscall(line);
	} else {
		print_unknown_command(line);
	}
}
