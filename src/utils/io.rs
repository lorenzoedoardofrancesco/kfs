//! # I/O Port Utilities
//!
//! Provides low-level functions for reading from and writing to I/O ports.
//! These functions are crucial in systems programming for interacting with
//! hardware devices. They use inline assembly and are marked as unsafe due
//! to their direct interaction with hardware.

use core::arch::asm;

/// Reads a byte from the specified I/O port.
///
/// # Safety
///
/// This function is unsafe because it performs direct I/O port access. Misuse can lead to
/// hardware faults and system instability. Always ensure that the port and value used are correct
/// and that the operation is safe in the current context.
///
/// # Arguments
///
/// * `port` - The I/O port address to read from.
///
/// # Returns
///
/// Returns the byte read from the specified port.
pub unsafe fn inb(port: u16) -> u8 {
	let value: u8;
	asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
	value
}

/// Writes a byte to the specified I/O port.
///
/// # Safety
///
/// As with `inb`, this function is unsafe for the same reasons - direct hardware access
/// and potential for misuse.
///
/// # Arguments
///
/// * `port` - The I/O port address to write to.
/// * `value` - The byte value to write to the port.
pub unsafe fn outb(port: u16, value: u8) {
	asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

/// Writes a word (2 bytes) to the specified I/O port.
///
/// # Safety
///
/// This function carries the same safety considerations as `inb` and `outb`.
///
/// # Arguments
///
/// * `port` - The I/O port address to write to.
/// * `value` - The word value to write to the port.
pub unsafe fn outw(port: u16, value: u16) {
	asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
}
