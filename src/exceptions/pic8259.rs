//! # Programmable Interrupt Controller (PIC)
//!
//! Provides functionality to interact with and manage the Programmable Interrupt Controllers (PICs)
//! in x86 systems. This module defines structures and methods for initializing the PICs, handling
//! interrupts, and sending end-of-interrupt commands. The PIC is crucial for managing hardware
//! interrupts in early x86-based systems.
//!
//! ## Overview
//!
//! The PICs is a pair of chips that are used to manage hardware interrupts in x86 systems. The PIC
//! is a legacy component that is no longer used in modern systems, but it is still present in
//! x86-based systems for backwards compatibility. The PIC is a pair of chips that are cascaded
//! together to provide 15 hardware interrupts. The PICs are cascaded together by connecting the
//! interrupt request (IRQ) 2 pin on the primary PIC to the interrupt acknowledge (INTA) pin on the
//! secondary PIC. The primary PIC is connected to the CPU via the IRQ 0 pin, and the secondary PIC
//! is connected to the CPU via the IRQ 8 pin. The PICs are initialized by sending a series of
//! commands to the PICs. The PICs are then configured to use the 8086/8088 mode, which is the
//! operating mode of the CPU.
//!
use crate::utils::io::{inb, outb};

const CMD_INIT: u8 = 0x11;
const CMD_END_OF_INTERRUPT: u8 = 0x20;
const MODE_8086: u8 = 0x01;

const PIC1_COMMAND: u8 = 0x20;
const PIC1_DATA: u8 = 0x21;
const PIC2_COMMAND: u8 = 0xa0;
const PIC2_DATA: u8 = 0xa1;

const WAIT_PORT: u8 = 0x80;

struct Pic {
	offset: u8,
	command: u8,
	data: u8,
}

impl Pic {
	fn handles_interrupt(&self, interrupt_id: u8) -> bool {
		self.offset <= interrupt_id && interrupt_id < self.offset + 8
	}

	unsafe fn end_of_interrupt(&mut self) {
		outb(self.command as u16, CMD_END_OF_INTERRUPT);
	}

	unsafe fn read_mask(&mut self) -> u8 {
		inb(self.data as u16)
	}

	unsafe fn write_mask(&mut self, mask: u8) {
		outb(self.data as u16, mask);
	}
}

/// Represents a pair of chained PICs.
pub struct ChainedPics {
	pics: [Pic; 2],
}

impl ChainedPics {
	/// Creates a new pair of chained PICs.
	pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
		ChainedPics {
			pics: [
				Pic {
					offset: offset1,
					command: PIC1_COMMAND,
					data: PIC1_DATA,
				},
				Pic {
					offset: offset2,
					command: PIC2_COMMAND,
					data: PIC2_DATA,
				},
			],
		}
	}

	pub const unsafe fn new_contiguous(primary_offset: u8) -> ChainedPics {
		Self::new(primary_offset, primary_offset + 8)
	}

	/// Initializes the PICs.
	pub unsafe fn initialize(&mut self) {
		let wait = || outb(WAIT_PORT as u16, 0);

		let saved_masks = self.read_masks();

		outb(self.pics[0].command as u16, CMD_INIT);
		wait();
		outb(self.pics[1].command as u16, CMD_INIT);
		wait();

		outb(self.pics[0].data as u16, self.pics[0].offset);
		wait();
		outb(self.pics[1].data as u16, self.pics[1].offset);
		wait();

		outb(self.pics[0].data as u16, 0x04);
		wait();
		outb(self.pics[1].data as u16, 0x02);
		wait();

		outb(self.pics[0].data as u16, MODE_8086);
		wait();
		outb(self.pics[1].data as u16, MODE_8086);
		wait();

		self.write_masks(saved_masks[0], saved_masks[1])
	}

	pub unsafe fn read_masks(&mut self) -> [u8; 2] {
		[self.pics[0].read_mask(), self.pics[1].read_mask()]
	}

	pub unsafe fn write_masks(&mut self, mask1: u8, mask2: u8) {
		self.pics[0].write_mask(mask1);
		self.pics[1].write_mask(mask2);
	}

	pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
		self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
	}

	pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
		if self.handles_interrupt(interrupt_id) {
			if self.pics[1].handles_interrupt(interrupt_id) {
				self.pics[1].end_of_interrupt();
			}
			self.pics[0].end_of_interrupt();
		}
	}
}
