use core::sync::atomic::{ AtomicBool, Ordering };
use spin::Mutex;
use crate::{ prompt, shell::HISTORY, shell::print_welcome_message };
use crate::video_graphics_array;

pub static KEYBOARD_INTERRUPT_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static LAST_SCANCODE: Mutex<u8> = Mutex::new(0);

static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static NUM_LOCK_PRESSED: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK_PRESSED: AtomicBool = AtomicBool::new(false);
//static ALT_GR_PRESSED: AtomicBool = AtomicBool::new(false);
static INSERT_PRESSED: AtomicBool = AtomicBool::new(false);
static FOREGROUND: bool = true;
static BACKGROUND: bool = false;

static QWERTY: bool = true;
static AZERTY: bool = false;
static KEYBOARD_LAYOUT: AtomicBool = AtomicBool::new(QWERTY);

pub fn process_keyboard_input() {
	static mut SCANCOD_BUFFER: [u8; 256] = [0; 256];
	static mut BUFFER_HEAD: usize = 0;
	static mut BUFFER_TAIL: usize = 0;

	if KEYBOARD_INTERRUPT_RECEIVED.load(Ordering::SeqCst) {
		unsafe {
			SCANCOD_BUFFER[BUFFER_HEAD] = *LAST_SCANCODE.lock();
			BUFFER_HEAD = (BUFFER_HEAD + 1) % SCANCOD_BUFFER.len();
		}
		KEYBOARD_INTERRUPT_RECEIVED.store(false, Ordering::SeqCst);
	}

	unsafe {
		while BUFFER_TAIL != BUFFER_HEAD {
			let scancode = SCANCOD_BUFFER[BUFFER_TAIL];
			BUFFER_TAIL = (BUFFER_TAIL + 1) % SCANCOD_BUFFER.len();

			update_modifier_state(scancode);
			let c = scancode_to_char(scancode);
			let ctrl = CTRL_PRESSED.load(Ordering::SeqCst);
			if c != '\0' && !ctrl {
				prompt::PROMPT.lock().insert_char(c as u8, INSERT_PRESSED.load(Ordering::SeqCst));
			}
		}
	}

	fn update_modifier_state(scancode: u8) {
		match scancode {
			0x2a | 0x36 => SHIFT_PRESSED.store(true, Ordering::SeqCst),
			0xaa | 0xb6 => SHIFT_PRESSED.store(false, Ordering::SeqCst),
			//0x1d => CTRL_PRESSED.store(true, Ordering::SeqCst),
			//0x9d => CTRL_PRESSED.store(false, Ordering::SeqCst),
			0x45 => {
				let num_lock = NUM_LOCK_PRESSED.load(Ordering::SeqCst);
				NUM_LOCK_PRESSED.store(!num_lock, Ordering::SeqCst);
			}
			0x3a => {
				let caps_lock = CAPS_LOCK_PRESSED.load(Ordering::SeqCst);
				CAPS_LOCK_PRESSED.store(!caps_lock, Ordering::SeqCst);
			}
			//0x38 => ALT_GR_PRESSED.store(true, Ordering::SeqCst),
			//0xb8 => ALT_GR_PRESSED.store(false, Ordering::SeqCst),
			0x52 => {
				let insert = INSERT_PRESSED.load(Ordering::SeqCst);
				INSERT_PRESSED.store(!insert, Ordering::SeqCst);
			}
			0x0e => prompt::backspace(),
			0x0f => prompt::tab(),
			0x4d => prompt::right_arrow(),
			0x4b => prompt::left_arrow(),
			0x47 => prompt::home(),
			0x4f => prompt::end(),
			0x48 => HISTORY.lock().scroll_up(),
			0x50 => HISTORY.lock().scroll_down(),
			0x53 => prompt::delete(),
			0x3b => video_graphics_array::change_display(0),
			0x3c => video_graphics_array::change_display(1),
			0x3d => video_graphics_array::change_display(2),
			0x3e => video_graphics_array::change_display(3),
			// 0x3f F5
			// 0x40 F6
			// 0x41 F7
			// 0x42 F8
			0x43 => print_welcome_message(),
			0x44 => change_keyboard_layout(),
			0x57 => video_graphics_array::change_color(FOREGROUND),
			0x58 => video_graphics_array::change_color(BACKGROUND),
			_ => (),
		}
	}

	fn change_keyboard_layout() {
		if KEYBOARD_LAYOUT.load(Ordering::SeqCst) == QWERTY {
			KEYBOARD_LAYOUT.store(AZERTY, Ordering::SeqCst);
		} else {
			KEYBOARD_LAYOUT.store(QWERTY, Ordering::SeqCst);
		}
	}

	fn scancode_to_char(scancode: u8) -> char {
		let shift = SHIFT_PRESSED.load(Ordering::SeqCst);
		//let ctrl = CTRL_PRESSED.load(Ordering::SeqCst);
		let num_lock = NUM_LOCK_PRESSED.load(Ordering::SeqCst);
		let caps_lock = CAPS_LOCK_PRESSED.load(Ordering::SeqCst);
		//let alt_gr = ALT_GR_PRESSED.load(Ordering::SeqCst);

		if KEYBOARD_LAYOUT.load(Ordering::SeqCst) == QWERTY {
			match scancode {
				0x01 => '\x1B',
				0x02 => if shift { '!' } else { '1' }
				0x03 => if shift { '@' } else { '2' }
				0x04 => if shift { '#' } else { '3' }
				0x05 => if shift { '$' } else { '4' }
				0x06 => if shift { '%' } else { '5' }
				0x07 => if shift { '^' } else { '6' }
				0x08 => if shift { '&' } else { '7' }
				0x09 => if shift { '*' } else { '8' }
				0x0a => if shift { '(' } else { '9' }
				0x0b => if shift { ')' } else { '0' }
				0x0c => if shift { '_' } else { '-' }
				0x0d => if shift { '+' } else { '=' }
				0x10 => if shift ^ caps_lock { 'Q' } else { 'q' }
				0x11 => if shift ^ caps_lock { 'W' } else { 'w' }
				0x12 => if shift ^ caps_lock { 'E' } else { 'e' }
				0x13 => if shift ^ caps_lock { 'R' } else { 'r' }
				0x14 => if shift ^ caps_lock { 'T' } else { 't' }
				0x15 => if shift ^ caps_lock { 'Y' } else { 'y' }
				0x16 => if shift ^ caps_lock { 'U' } else { 'u' }
				0x17 => if shift ^ caps_lock { 'I' } else { 'i' }
				0x18 => if shift ^ caps_lock { 'O' } else { 'o' }
				0x19 => if shift ^ caps_lock { 'P' } else { 'p' }
				0x1a => if shift { '{' } else { '[' }
				0x1b => if shift { '}' } else { ']' }
				0x1c => '\n',
				0x1e => if shift ^ caps_lock { 'A' } else { 'a' }
				0x1f => if shift ^ caps_lock { 'S' } else { 's' }
				0x20 => if shift ^ caps_lock { 'D' } else { 'd' }
				0x21 => if shift ^ caps_lock { 'F' } else { 'f' }
				0x22 => if shift ^ caps_lock { 'G' } else { 'g' }
				0x23 => if shift ^ caps_lock { 'H' } else { 'h' }
				0x24 => if shift ^ caps_lock { 'J' } else { 'j' }
				0x25 => if shift ^ caps_lock { 'K' } else { 'k' }
				0x26 => if shift ^ caps_lock { 'L' } else { 'l' }
				0x27 => if shift { ':' } else { ';' }
				0x28 => if shift { '"' } else { '\'' }
				0x29 => if shift { '~' } else { '`' }
				0x2b => if shift { '|' } else { '\\' }
				0x2c => if shift ^ caps_lock { 'Z' } else { 'z' }
				0x2d => if shift ^ caps_lock { 'X' } else { 'x' }
				0x2e => if shift ^ caps_lock { 'C' } else { 'c' }
				0x2f => if shift ^ caps_lock { 'V' } else { 'v' }
				0x30 => if shift ^ caps_lock { 'B' } else { 'b' }
				0x31 => if shift ^ caps_lock { 'N' } else { 'n' }
				0x32 => if shift ^ caps_lock { 'M' } else { 'm' }
				0x33 => if shift { '<' } else { ',' }
				0x34 => if shift { '>' } else { '.' }
				0x35 => '/',
				0x37 => '*',
				0x39 => ' ',
				0x47 => if num_lock { '7' } else { '\0' }
				0x48 => if num_lock { '8' } else { '\0' }
				0x49 => if num_lock { '9' } else { '\0' }
				0x4a => '-',
				0x4b => if num_lock { '4' } else { '\0' }
				0x4c => if num_lock { '5' } else { '\0' }
				0x4d => if num_lock { '6' } else { '\0' }
				0x4e => '+',
				0x4f => if num_lock { '1' } else { '\0' }
				0x50 => if num_lock { '2' } else { '\0' }
				0x51 => if num_lock { '3' } else { '\0' }
				0x52 => if num_lock { '0' } else { '\0' }
				0x53 => if num_lock { '.' } else { '\0' }
				_ => '\0',
			}
		} else {
			match scancode {
				0x01 => '\x1B',
				0x02 => if shift ^ caps_lock { '1' } else { '&' }
				0x03 => if shift ^ caps_lock { '2' } else { '\0' }
				0x04 => if shift ^ caps_lock { '3' } else { '"' }
				0x05 => if shift ^ caps_lock { '4' } else { '\'' }
				0x06 => if shift ^ caps_lock { '5' } else { '(' }
				0x07 => if shift ^ caps_lock { '6' } else { '-' }
				0x08 => if shift ^ caps_lock { '7' } else { '\0' }
				0x09 => if shift ^ caps_lock { '8' } else { '_' }
				0x0a => if shift ^ caps_lock { '9' } else { '\0' }
				0x0b => if shift ^ caps_lock { '0' } else { '\0' }
				0x0c => if shift ^ caps_lock { ')' } else { '\0' }
				0x0d => if shift ^ caps_lock { '=' } else { '+' }
				0x10 => if shift ^ caps_lock { 'A' } else { 'a' }
				0x11 => if shift ^ caps_lock { 'Z' } else { 'z' }
				0x12 => if shift ^ caps_lock { 'E' } else { 'e' }
				0x13 => if shift ^ caps_lock { 'R' } else { 'r' }
				0x14 => if shift ^ caps_lock { 'T' } else { 't' }
				0x15 => if shift ^ caps_lock { 'Y' } else { 'y' }
				0x16 => if shift ^ caps_lock { 'U' } else { 'u' }
				0x17 => if shift ^ caps_lock { 'I' } else { 'i' }
				0x18 => if shift ^ caps_lock { 'O' } else { 'o' }
				0x19 => if shift ^ caps_lock { 'P' } else { 'p' }
				0x1a => if shift ^ caps_lock { '^' } else { '\0' }
				0x1b => if shift ^ caps_lock { '\0' } else { '$' }
				0x1c => '\n',
				0x1e => if shift ^ caps_lock { 'Q' } else { 'q' }
				0x1f => if shift ^ caps_lock { 'S' } else { 's' }
				0x20 => if shift ^ caps_lock { 'D' } else { 'd' }
				0x21 => if shift ^ caps_lock { 'F' } else { 'f' }
				0x22 => if shift ^ caps_lock { 'G' } else { 'g' }
				0x23 => if shift ^ caps_lock { 'H' } else { 'h' }
				0x24 => if shift ^ caps_lock { 'J' } else { 'j' }
				0x25 => if shift ^ caps_lock { 'K' } else { 'k' }
				0x26 => if shift ^ caps_lock { 'L' } else { 'l' }
				0x27 => if shift ^ caps_lock { 'M' } else { 'm' }
				0x28 => if shift ^ caps_lock { '%' } else { '\0' }
				0x29 => '\0',
				0x2b => if shift ^ caps_lock { '\0' } else { '*' }
				0x2c => if shift ^ caps_lock { 'W' } else { 'w' }
				0x2d => if shift ^ caps_lock { 'X' } else { 'x' }
				0x2e => if shift ^ caps_lock { 'C' } else { 'c' }
				0x2f => if shift ^ caps_lock { 'V' } else { 'v' }
				0x30 => if shift ^ caps_lock { 'B' } else { 'b' }
				0x31 => if shift ^ caps_lock { 'N' } else { 'n' }
				0x32 => if shift ^ caps_lock { '?' } else { ',' }
				0x33 => if shift ^ caps_lock { '.' } else { ';' }
				0x34 => if shift ^ caps_lock { '/' } else { ':' }
				0x35 => '/',
				0x37 => '*',
				0x39 => ' ',
				0x47 => if num_lock { '7' } else { '\0' }
				0x48 => if num_lock { '8' } else { '\0' }
				0x49 => if num_lock { '9' } else { '\0' }
				0x4a => '-',
				0x4b => if num_lock { '4' } else { '\0' }
				0x4c => if num_lock { '5' } else { '\0' }
				0x4d => if num_lock { '6' } else { '\0' }
				0x4e => '+',
				0x4f => if num_lock { '1' } else { '\0' }
				0x50 => if num_lock { '2' } else { '\0' }
				0x51 => if num_lock { '3' } else { '\0' }
				0x52 => if num_lock { '0' } else { '\0' }
				0x53 => if num_lock { '.' } else { '\0' }
				_ => '\0',
			}
		}
	}
}
