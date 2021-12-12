use lazy_static::lazy_static;
use spin::Mutex;


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::text::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
		use x86_64::instructions::interrupts;

		let mut writer = WRITER.lock();

    interrupts::without_interrupts(|| {
        writer.write_fmt(args).unwrap();
    });
}

pub const DEFAULT_CLR: u8 = 0x0F;
pub const PANIC_CLR: u8 = 0x4F;

lazy_static! {
	pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
	pub static ref SCREEN_CLR: Mutex<u8> = Mutex::new(DEFAULT_CLR);
}


pub struct Writer {
	slice: &'static mut [u8],
	pub cursor: usize
}

impl Writer {
	pub fn new() -> Self {
		Writer { cursor: 0, slice: unsafe { core::slice::from_raw_parts_mut(0xb8000 as *mut u8, 4000) } }
	}
	pub fn display(&mut self, to_display: &str, attr: u8) {
		for chr in to_display.bytes() {
			if self.cursor+1 > 80*25 {
				self.clear(0);
			}
			if chr == b'\n' {
				for _ in 0..80-(self.cursor%80) {
					self.display(" ", *SCREEN_CLR.lock());
				}
				continue
			}
		    	self.slice[self.cursor * 2] = chr;
		    	self.slice[self.cursor * 2 + 1] = attr;
			self.cursor += 1;
    }
	}
	pub fn display_overwrite(&mut self, to_display: &str, attr: u8) {
		let last = self.cursor;
		*SCREEN_CLR.lock() = attr;
		write!(self, "{}", to_display);
		*SCREEN_CLR.lock() = DEFAULT_CLR;
		self.cursor = last;
	}
	pub fn blink(&mut self) {
		match self.slice[self.cursor * 2 + 1] & 0xF0 {
			0x00 => self.slice[self.cursor * 2 + 1] = 0xF0,
			0xF0 => self.slice[self.cursor * 2 + 1] = 0x0F,
			any => {}
		}
	}
	pub fn clear(&mut self, colour: u8) {
		for i in 0..80*25 {
						self.slice[i * 2] = b' ';
						self.slice[i * 2 + 1] = colour;
				}
				self.cursor = 0;
	}
	pub fn clear_last(&mut self) {
		if self.cursor != 0 {
			self.cursor -= 1;
		}
		if self.slice[self.cursor * 2 + 1] == PANIC_CLR {
			self.cursor += 1;
			// Stops the cursor from going before a specified colour
			return;
		}
		self.slice[self.cursor * 2] = b' ';
		self.slice[self.cursor * 2 + 1] = SCREEN_CLR.lock().clone();
	}
}

use core::fmt::Write;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
			let lock = SCREEN_CLR.lock().clone();
        self.display(s, lock);
				drop(lock);
        Ok(())
    }
}

pub fn display(to_display: &str, attr: u8) {
		let slice: *mut u8 = 0xb8000 as *mut u8;

    for (idx, chr) in to_display.bytes().enumerate() {
        unsafe {
		    	slice.offset(idx as isize * 2).write_volatile(chr);
		    	slice.offset(idx as isize * 2 + 1).write_volatile(attr);
	    	}
    }
}