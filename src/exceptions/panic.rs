use crate::exceptions::interrupts::InterruptStackFrame;
use crate::shell::prints::PrintStackMode;
use crate::utils::debug::LogLevel;
use crate::utils::librs::{hexdump, hlt};
use core::arch::asm;
use core::fmt::Display;

const STACK_DUMP_SIZE: usize = 1024;
static mut STACK_DUMP: [u8; STACK_DUMP_SIZE] = [0; STACK_DUMP_SIZE];

pub fn save_stack() {
	let stack_pointer: usize;
	unsafe {
		asm!("mov {}, esp", out(reg) stack_pointer, options(nostack, preserves_flags));

		let buffer_ptr = STACK_DUMP.as_mut_ptr();

		buffer_ptr.copy_from(stack_pointer as *const u8, STACK_DUMP_SIZE);
	}
}

pub fn clean_registers() {
	unsafe {
		asm!(
			"xor eax, eax",
			"xor ebx, ebx",
			"xor ecx, ecx",
			"xor edx, edx",
			"xor esi, esi",
			"xor edi, edi",
			"xor ebp, ebp",
			options(nostack, preserves_flags)
		);
	}
}

pub fn verify_clean_registers() {
	unsafe {
		let (eax, ebx, ecx, edx, esi, edi, ebp): (usize, usize, usize, usize, usize, usize, usize);

		asm!(
			"mov {}, eax",
			"mov {}, ebx",
			"mov {}, ecx",
			"mov {}, edx",
			"mov {}, esi",
			"mov {}, edi",
			"mov {}, ebp",
			out(reg) eax, out(reg) ebx, out(reg) ecx,
			out(reg) edx, out(reg) esi, out(reg) edi,
			out(reg) ebp,
			options(nostack, preserves_flags)
		);

		assert_eq!(eax, 0, "EAX not cleaned");
		assert_eq!(ebx, 0, "EBX not cleaned");
		assert_eq!(ecx, 0, "ECX not cleaned");
		assert_eq!(edx, 0, "EDX not cleaned");
		assert_eq!(esi, 0, "ESI not cleaned");
		assert_eq!(edi, 0, "EDI not cleaned");
		assert_eq!(ebp, 0, "EBP not cleaned");
	}
}

pub fn handle_panic<D: Display>(info: &D, stack_frame: Option<&InterruptStackFrame>) -> ! {
	save_stack();
	clean_registers();
	verify_clean_registers();

	log!(LogLevel::Panic, "{}", info);
	println!("{}", info);

	if let Some(frame) = stack_frame {
		log!(LogLevel::Panic, "{:#?}", frame);
		println!("{:#?}", frame);
	}

	unsafe {
		let stack_start_address = STACK_DUMP.as_ptr();
		log!(LogLevel::Info, "Stack dump at {:#x}", stack_start_address as usize);
		hexdump(stack_start_address as usize, STACK_DUMP_SIZE, PrintStackMode::Serial);
	}

	println!("See serial output for more information.");

	loop {
		hlt();
	}
}
