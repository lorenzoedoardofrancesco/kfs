//! # Shell History Module
//!
//! This module provides functionality for maintaining a command history in a shell-like interface.
//! It includes the implementation of a history buffer that stores previously entered commands
//! and allows the user to scroll through them. This feature enhances the user experience by
//! enabling easy recall and modification of previous commands.

use crate::shell::builtins::{MAX_HISTORY_LINES, MAX_LINE_LENGTH};
use crate::utils::librs::{array_cmp, array_to_str, str_to_array};
use crate::vga::prompt::{PROMPT, self};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
	/// Mutex-protected global instance of the shell history.
	pub static ref HISTORY: Mutex<History> = Mutex::new(History::new());
}

pub type Line = [u8; MAX_LINE_LENGTH];

/// Structure representing the shell command history.
///
/// Maintains a buffer of previously entered commands, along with indices for adding and
/// retrieving commands.
pub struct History {
	buffer: [Line; MAX_HISTORY_LINES],
	last_input: [u8; MAX_LINE_LENGTH],
	index: usize,
	add_index: usize,
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

	pub fn add(&mut self, line: &str) {
		let line_u8 = str_to_array(line);

		self.index = self.add_index;
		if array_cmp(&line_u8, &self.last_input) {
			return;
		}
		self.update_history(line_u8);
	}

	pub fn update_history(&mut self, line_u8: Line) {
		self.buffer[self.add_index] = line_u8;
		self.last_input = line_u8;
		self.add_index = (self.add_index + 1) % MAX_HISTORY_LINES;
		self.index = self.add_index;
		self.buffer[self.index] = [0; MAX_LINE_LENGTH];
	}

	pub fn get(&self, index: usize) -> &Line {
		&self.buffer[index % MAX_HISTORY_LINES]
	}

	pub fn print(&self) {
		for line in self.buffer.iter().filter(|l| l[0] != 0) {
			println!("{}", array_to_str(line));
		}
	}

	pub fn print_prompt(&self) {
		prompt::init();
		PROMPT
			.lock()
			.insert_string(array_to_str(self.get(self.index)));
	}

	pub fn scroll_up(&mut self) {
		let original_index = self.index;

		if self.index == 0 {
			self.index = MAX_HISTORY_LINES - 1;
		} else {
			self.index -= 1;
		}

		if self.buffer[self.index][0] == 0 {
			self.index = original_index;
			return;
		}

		self.print_prompt();
	}

	pub fn scroll_down(&mut self) {
		if self.buffer[self.index][0] == 0 {
			return;
		}
		if self.index == MAX_HISTORY_LINES - 1 {
			self.index = 0;
		} else {
			self.index += 1;
		}

		self.print_prompt();
	}
}
