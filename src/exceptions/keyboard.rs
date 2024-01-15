use crate::shell;
use crate::shell::history::HISTORY;
use crate::shell::prints::print_welcome_message;
use crate::vga::parrot::PARROT_ACTIVATED;
use crate::vga::video_graphics_array::WRITER;
use crate::vga::{prompt, video_graphics_array};
use core::sync::atomic::{AtomicBool, Ordering};

pub static KEYBOARD_INTERRUPT_RECEIVED: AtomicBool = AtomicBool::new(false);
static SERIAL_SCREEN: AtomicBool = AtomicBool::new(false);

static ESCAPE_PREFIX_RECEIVED: AtomicBool = AtomicBool::new(false);
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static NUM_LOCK_PRESSED: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK_PRESSED: AtomicBool = AtomicBool::new(false);
static ALT_GR_PRESSED: AtomicBool = AtomicBool::new(false);
static INSERT_PRESSED: AtomicBool = AtomicBool::new(false);
static FOREGROUND: bool = true;
static BACKGROUND: bool = false;

static QWERTY: bool = true;
static AZERTY: bool = false;
static KEYBOARD_LAYOUT: AtomicBool = AtomicBool::new(QWERTY);

pub static mut SCANCODE_BUFFER: [u8; 128] = [0; 128];
pub static mut BUFFER_HEAD: usize = 0;
static mut BUFFER_TAIL: usize = 0;
static ESCAPE_PREFIX: u8 = 0xe0;

pub fn process_keyboard_input() {
	if KEYBOARD_INTERRUPT_RECEIVED.load(Ordering::SeqCst) {
		KEYBOARD_INTERRUPT_RECEIVED.store(false, Ordering::SeqCst);
	} else {
		return;
	}

	let parrot_activated = PARROT_ACTIVATED.load(Ordering::SeqCst);
	if parrot_activated {
		WRITER.lock().show_cursor();
		PARROT_ACTIVATED.store(false, Ordering::SeqCst);
		prompt::init();
	}

	let serial_screen = SERIAL_SCREEN.load(Ordering::SeqCst);
	if serial_screen {
		WRITER.lock().show_cursor();
		SERIAL_SCREEN.store(false, Ordering::SeqCst);
		video_graphics_array::change_display(0);
	}

	unsafe {
		while BUFFER_TAIL != BUFFER_HEAD {
			let scancode = SCANCODE_BUFFER[BUFFER_TAIL];
			BUFFER_TAIL = (BUFFER_TAIL + 1) % SCANCODE_BUFFER.len();

			if scancode == ESCAPE_PREFIX {
				ESCAPE_PREFIX_RECEIVED.store(true, Ordering::SeqCst);
				continue;
			}

			let escape_prefix = ESCAPE_PREFIX_RECEIVED.load(Ordering::SeqCst);
			let mut c: u8 = 0;

			if !escape_prefix {
				c = scancode_to_char(scancode);
			}
			if c == b'\0' {
				update_modifier_state(scancode);
				continue;
			}
			let ctrl = CTRL_PRESSED.load(Ordering::SeqCst);
			if !ctrl {
				prompt::PROMPT
					.lock()
					.insert_char(c as u8, INSERT_PRESSED.load(Ordering::SeqCst));
			} else if ctrl {
				if c == b'l' {
					shell::builtins::clear();
					prompt::init();
				} else if c == b'c' {
					prompt::init();
				}
			}
		}
	}
}

fn update_modifier_state(scancode: u8) {
	let escape_prefix = ESCAPE_PREFIX_RECEIVED.load(Ordering::SeqCst);

	if escape_prefix {
		match scancode {
			0x1c => prompt::enter(),
			0x1d => CTRL_PRESSED.store(true, Ordering::SeqCst),
			0x9d => CTRL_PRESSED.store(false, Ordering::SeqCst),
			0x37 => print!("*PRTSC*"),
			0x38 => ALT_GR_PRESSED.store(true, Ordering::SeqCst),
			0xb8 => ALT_GR_PRESSED.store(false, Ordering::SeqCst),
			0x47 => prompt::home(),
			0x48 => HISTORY.lock().scroll_up(),
			0x49 => HISTORY.lock().scroll_up(),
			0x4b => prompt::left_arrow(),
			0x4d => prompt::right_arrow(),
			0x4f => prompt::end(),
			0x50 => HISTORY.lock().scroll_down(),
			0x51 => HISTORY.lock().scroll_down(),
			0x52 => {
				let insert = INSERT_PRESSED.load(Ordering::SeqCst);
				INSERT_PRESSED.store(!insert, Ordering::SeqCst)
			}
			0x53 => prompt::delete(),

			_ => (),
		}
		ESCAPE_PREFIX_RECEIVED.store(false, Ordering::SeqCst);
	} else {
		match scancode {
			0x2a | 0x36 => SHIFT_PRESSED.store(true, Ordering::SeqCst),
			0xaa | 0xb6 => SHIFT_PRESSED.store(false, Ordering::SeqCst),
			0x1d => CTRL_PRESSED.store(true, Ordering::SeqCst),
			0x9d => CTRL_PRESSED.store(false, Ordering::SeqCst),
			0x45 => {
				let num_lock = NUM_LOCK_PRESSED.load(Ordering::SeqCst);
				NUM_LOCK_PRESSED.store(!num_lock, Ordering::SeqCst);
			}
			0x3a => {
				let caps_lock = CAPS_LOCK_PRESSED.load(Ordering::SeqCst);
				CAPS_LOCK_PRESSED.store(!caps_lock, Ordering::SeqCst);
			}
			0x0e => prompt::backspace(),
			0x0f => prompt::tab(),
			0x4d => prompt::right_arrow(),
			0x4b => prompt::left_arrow(),
			0x4f => prompt::end(),
			0x48 => HISTORY.lock().scroll_up(),
			0x50 => HISTORY.lock().scroll_down(),
			0x3b => video_graphics_array::change_display(0),
			0x3c => video_graphics_array::change_display(1),
			0x3d => video_graphics_array::change_display(2),
			0x3e => video_graphics_array::change_display(3),
			0x3f => video_graphics_array::change_display(4),
			// 0x40 F6
			// 0x41 F7
			// 0x42 F8
			0xbf => {
				SERIAL_SCREEN.store(true, Ordering::SeqCst);
				WRITER.lock().hide_cursor();
			}
			0xc2 => {
				PARROT_ACTIVATED.store(true, Ordering::SeqCst);
				WRITER.lock().hide_cursor();
			}
			0x43 => print_welcome_message(),
			0x44 => change_keyboard_layout(),
			0x47 => prompt::home(),
			0x52 => {
				let insert = INSERT_PRESSED.load(Ordering::SeqCst);
				INSERT_PRESSED.store(!insert, Ordering::SeqCst)
			}
			0x53 => prompt::delete(),
			0x57 => video_graphics_array::change_color(FOREGROUND),
			0x58 => video_graphics_array::change_color(BACKGROUND),
			_ => (),
		}
	}
}

fn change_keyboard_layout() {
	if KEYBOARD_LAYOUT.load(Ordering::SeqCst) == QWERTY {
		KEYBOARD_LAYOUT.store(AZERTY, Ordering::SeqCst);
	} else {
		KEYBOARD_LAYOUT.store(QWERTY, Ordering::SeqCst);
	}
}

#[rustfmt::skip]
fn scancode_to_char(scancode: u8) -> u8 {
	let shift = SHIFT_PRESSED.load(Ordering::SeqCst);
	let num_lock = NUM_LOCK_PRESSED.load(Ordering::SeqCst);
	let caps_lock = CAPS_LOCK_PRESSED.load(Ordering::SeqCst);
	let alt_gr = ALT_GR_PRESSED.load(Ordering::SeqCst);

	if KEYBOARD_LAYOUT.load(Ordering::SeqCst) == QWERTY {
		match scancode {
			0x01 => b'\x1B',
			0x02 => if shift { b'!' } else { b'1' }
			0x03 => if shift { b'@' } else { b'2' }
			0x04 => if shift { b'#' } else { b'3' }
			0x05 => if shift { b'$' } else { b'4' }
			0x06 => if shift { b'%' } else { b'5' }
			0x07 => if shift { b'^' } else { b'6' }
			0x08 => if shift { b'&' } else { b'7' }
			0x09 => if shift { b'*' } else { b'8' }
			0x0a => if shift { b'(' } else { b'9' }
			0x0b => if shift { b')' } else { b'0' }
			0x0c => if shift { b'_' } else { b'-' }
			0x0d => if shift { b'+' } else { b'=' }
			0x10 => if shift ^ caps_lock { b'Q' } else { b'q' }
			0x11 => if shift ^ caps_lock { b'W' } else { b'w' }
			0x12 => if shift ^ caps_lock { b'E' } else { b'e' }
			0x13 => if shift ^ caps_lock { b'R' } else { b'r' }
			0x14 => if shift ^ caps_lock { b'T' } else { b't' }
			0x15 => if shift ^ caps_lock { b'Y' } else { b'y' }
			0x16 => if shift ^ caps_lock { b'U' } else { b'u' }
			0x17 => if shift ^ caps_lock { b'I' } else { b'i' }
			0x18 => if shift ^ caps_lock { b'O' } else { b'o' }
			0x19 => if shift ^ caps_lock { b'P' } else { b'p' }
			0x1a => if shift { b'{' } else { b'[' }
			0x1b => if shift { b'}' } else { b']' }
			0x1c => b'\n',
			0x1e => if shift ^ caps_lock { b'A' } else { b'a' }
			0x1f => if shift ^ caps_lock { b'S' } else { b's' }
			0x20 => if shift ^ caps_lock { b'D' } else { b'd' }
			0x21 => if shift ^ caps_lock { b'F' } else { b'f' }
			0x22 => if shift ^ caps_lock { b'G' } else { b'g' }
			0x23 => if shift ^ caps_lock { b'H' } else { b'h' }
			0x24 => if shift ^ caps_lock { b'J' } else { b'j' }
			0x25 => if shift ^ caps_lock { b'K' } else { b'k' }
			0x26 => if shift ^ caps_lock { b'L' } else { b'l' }
			0x27 => if shift { b':' } else { b';' }
			0x28 => if shift { b'"' } else { b'\'' }
			0x29 => if shift { b'~' } else { b'`' }
			0x2b => if shift { b'|' } else { b'\\' }
			0x2c => if shift ^ caps_lock { b'Z' } else { b'z' }
			0x2d => if shift ^ caps_lock { b'X' } else { b'x' }
			0x2e => if shift ^ caps_lock { b'C' } else { b'c' }
			0x2f => if shift ^ caps_lock { b'V' } else { b'v' }
			0x30 => if shift ^ caps_lock { b'B' } else { b'b' }
			0x31 => if shift ^ caps_lock { b'N' } else { b'n' }
			0x32 => if shift ^ caps_lock { b'M' } else { b'm' }
			0x33 => if shift { b'<' } else { b',' }
			0x34 => if shift { b'>' } else { b'.' }
			0x35 => b'/',
			0x37 => b'*',
			0x39 => b' ',
			0x47 => if num_lock { b'7' } else { b'\0' }
			0x48 => if num_lock { b'8' } else { b'\0' }
			0x49 => if num_lock { b'9' } else { b'\0' }
			0x4a => b'-',
			0x4b => if num_lock { b'4' } else { b'\0' }
			0x4c => if num_lock { b'5' } else { b'\0' }
			0x4d => if num_lock { b'6' } else { b'\0' }
			0x4e => b'+',
			0x4f => if num_lock { b'1' } else { b'\0' }
			0x50 => if num_lock { b'2' } else { b'\0' }
			0x51 => if num_lock { b'3' } else { b'\0' }
			0x52 => if num_lock { b'0' } else { b'\0' }
			0x53 => if num_lock { b'.' } else { b'\0' }
			_ => b'\0',
		}
	} else {
		match scancode {
			0x01 => b'\x1B',
			0x02 => if shift { b'1' } else { b'&' }
			0x03 => if shift { b'2' } else if alt_gr { b'~' } else if caps_lock { 0x0f } else { 0x03 }
			0x04 => if shift { b'3' } else if alt_gr { b'#' } else { b'"' }
			0x05 => if shift { b'4' } else if alt_gr { b'{' } else { b'\'' }
			0x06 => if shift { b'5' } else if alt_gr { b'[' } else { b'(' }
			0x07 => if shift { b'6' } else if alt_gr { b'|' } else { b'-' }
			0x08 => if shift { b'7' } else if alt_gr { b'`' } else { 0x0b }
			0x09 => if shift { b'8' } else if alt_gr { b'\\' } else { b'_' }
			0x0a => if shift { b'9' } else if alt_gr { b'^' } else if caps_lock { 0x01 } else { 0x07 }
			0x0b => if shift { b'0' } else if alt_gr { b'@' } else { 0x06 }
			0x0c => if shift { 0x18 } else if alt_gr { b']' } else { b')' }
			0x0d => if shift { b'+' } else if alt_gr { b'}' } else { b'=' }
			0x10 => if shift ^ caps_lock { b'A' } else { b'a' }
			0x11 => if shift ^ caps_lock { b'Z' } else { b'z' }
			0x12 => if shift ^ caps_lock { b'E' } else { b'e' }
			0x13 => if shift ^ caps_lock { b'R' } else { b'r' }
			0x14 => if shift ^ caps_lock { b'T' } else { b't' }
			0x15 => if shift ^ caps_lock { b'Y' } else { b'y' }
			0x16 => if shift ^ caps_lock { b'U' } else { b'u' }
			0x17 => if shift ^ caps_lock { b'I' } else { b'i' }
			0x18 => if shift ^ caps_lock { b'O' } else { b'o' }
			0x19 => if shift ^ caps_lock { b'P' } else { b'p' }
			0x1a => if shift { b'\0' } else {  b'^' }
			0x1b => if shift { 0x16 } else { b'$' }
			0x1c => b'\n',
			0x1e => if shift ^ caps_lock { b'Q' } else { b'q' }
			0x1f => if shift ^ caps_lock { b'S' } else { b's' }
			0x20 => if shift ^ caps_lock { b'D' } else { b'd' }
			0x21 => if shift ^ caps_lock { b'F' } else { b'f' }
			0x22 => if shift ^ caps_lock { b'G' } else { b'g' }
			0x23 => if shift ^ caps_lock { b'H' } else { b'h' }
			0x24 => if shift ^ caps_lock { b'J' } else { b'j' }
			0x25 => if shift ^ caps_lock { b'K' } else { b'k' }
			0x26 => if shift ^ caps_lock { b'L' } else { b'l' }
			0x27 => if shift ^ caps_lock { b'M' } else { b'm' }
			0x28 => if shift { b'%' } else { 0x13 }
			0x29 => 0x19,
			0x2b => if shift { 0x17 } else { b'*' }
			0x2c => if shift ^ caps_lock { b'W' } else { b'w' }
			0x2d => if shift ^ caps_lock { b'X' } else { b'x' }
			0x2e => if shift ^ caps_lock { b'C' } else { b'c' }
			0x2f => if shift ^ caps_lock { b'V' } else { b'v' }
			0x30 => if shift ^ caps_lock { b'B' } else { b'b' }
			0x31 => if shift ^ caps_lock { b'N' } else { b'n' }
			0x32 => if shift { b'?' } else { b',' }
			0x33 => if shift { b'.' } else { b';' }
			0x34 => if shift { b'/' } else { b':' }
			0x35 => if shift { 0x1a } else { b'!' }
			0x37 => b'*',
			0x39 => b' ',
			0x47 => if num_lock { b'7' } else { b'\0' }
			0x48 => if num_lock { b'8' } else { b'\0' }
			0x49 => if num_lock { b'9' } else { b'\0' }
			0x4a => b'-',
			0x4b => if num_lock { b'4' } else { b'\0' }
			0x4c => if num_lock { b'5' } else { b'\0' }
			0x4d => if num_lock { b'6' } else { b'\0' }
			0x4e => b'+',
			0x4f => if num_lock { b'1' } else { b'\0' }
			0x50 => if num_lock { b'2' } else { b'\0' }
			0x51 => if num_lock { b'3' } else { b'\0' }
			0x52 => if num_lock { b'0' } else { b'\0' }
			0x53 => if num_lock { b'.' } else { b'\0' }
			_ => b'\0',
		}
	}
}
