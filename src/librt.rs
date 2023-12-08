use crate::console::{ putchar, new_line, update_cursor, Color };

#[macro_export]
macro_rules! printf {
    (
        $fmt:expr $(
            ,
            $($arg:tt)+
        )?
    ) => {
        {
		use core::fmt::Write;
		
		struct Writer;

		impl core::fmt::Write for Writer {
			fn write_str(&mut self, s: &str) -> core::fmt::Result {
				putstr(s, Color::White);
				Ok(())
			}
		}

		let mut writer = Writer;
		let _ = write!(&mut writer, $fmt $(, $($arg)+)?);
        }
    };
}

pub fn putstr(s: &str, color: Color) {
    for byte in s.bytes() {
        match byte {
            b'\n' => new_line(),
            byte => putchar(byte, color),
        }
    }
    update_cursor();
}
