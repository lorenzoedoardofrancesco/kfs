use crate::utils::debug::LogLevel;

#[derive(Debug, Clone, Copy)]
pub enum SyscallNumber {
	Exit = 0,
	Write = 1,
	Read = 2,
}

impl From<u32> for SyscallNumber {
	fn from(num: u32) -> Self {
		match num {
			0 => SyscallNumber::Exit,
			1 => SyscallNumber::Write,
			2 => SyscallNumber::Read,
			_ => panic!("Invalid syscall number"),
		}
	}
}

pub struct SyscallParameters<'a> {
	regs: &'a mut GeneralRegs,
}

type SyscallFn = fn(&mut SyscallParameters);

pub struct SyscallEntry {
	func: SyscallFn,
}

static SYSCALL_TABLE: [SyscallEntry; 3] = [
	SyscallEntry { func: sys_exit },
	SyscallEntry { func: sys_write },
	SyscallEntry { func: sys_read },
];

#[repr(C)]
pub struct GeneralRegs {
	pub eax: u32,
	pub ebx: u32,
	pub ecx: u32,
	pub edx: u32,
	pub esi: u32,
	pub edi: u32,
	pub ebp: u32,
}

pub fn syscall(regs: &mut GeneralRegs) {
	let num = SyscallNumber::from(regs.eax);
	if num as usize >= SYSCALL_TABLE.len() {
		log!(LogLevel::Warning, "Syscall {:#x} not handled", regs.eax);
		return;
	}

	let syscall_func = SYSCALL_TABLE[num as usize].func;
	let mut params = SyscallParameters { regs: regs };
	syscall_func(&mut params);

	log!(
		LogLevel::Debug,
		"INT 0x80 (syscall) called with eax {}",
		regs.eax
	);
}

fn sys_exit(params: &mut SyscallParameters) {
	log!(
		LogLevel::Debug,
		"Syscall exit called with code {}",
		params.regs.ebx
	);
}

fn sys_write(params: &mut SyscallParameters) {
	let fd = params.regs.ebx; // File descriptor
	let buf_ptr = params.regs.ecx as *const u8; // Buffer pointer
	let count = params.regs.edx as usize; // Number of bytes to write

	if fd == 1 {
		// Assuming '1' is standard output
		// Directly iterate over the memory location
		for i in 0..count {
			let char_byte = unsafe { *buf_ptr.add(i) };
			print!("{}", char_byte as char);
		}
		println!(); // Newline for ending the output
	} else {
		log!(LogLevel::Error, "Unsupported file descriptor: {}", fd);
	}
}

fn sys_read(params: &mut SyscallParameters) {
	// In a real scenario, this would involve waiting for input
	// In this simplified version, just log the read attempt
	let fd = params.regs.ebx; // File descriptor
	let buf_ptr = params.regs.ecx; // Buffer pointer
	let count = params.regs.edx; // Number of bytes to read

	log!(
		LogLevel::Debug,
		"Syscall read called with fd {}, buf {:?}, count {}",
		fd,
		buf_ptr,
		count
	);
}
