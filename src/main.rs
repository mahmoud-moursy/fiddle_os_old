#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]


extern crate bootloader;

use bootloader::{ BootInfo, entry_point };

use core::any::Any;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;

pub mod interrupts;
pub mod text;
pub mod gdt;
pub mod driver;
pub mod memory;

use x86_64::structures::idt::InterruptStackFrame;
use driver::keyboard::*;
use text::*;
use pc_keyboard::*;


#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
	let mut writer = text::Writer::new();
	writer.clear(text::PANIC_CLR);
	writer.display("Kernel panic: ", text::PANIC_CLR);
	*text::SCREEN_CLR.lock() = text::PANIC_CLR;
	write!(writer, "{}", _info).unwrap();

	loop {
		x86_64::instructions::hlt();
	}
}

entry_point!(kern_start);

use lazy_static::lazy_static;

pub struct App {
	data: &'static [&'static (dyn Any + Send + Sync)],
	instruction: fn()
}

use spin::Mutex;

lazy_static! {
	pub static ref APP_SPACE: Mutex<App> = Mutex::new(App { data: &[], instruction: || x86_64::instructions::hlt() });
}

#[no_mangle]
pub fn kern_start(_boot_info: &'static BootInfo) -> ! {
	gdt::init();
	interrupts::init_idt();

	unsafe { interrupts::PICS.lock().initialize() };
	x86_64::instructions::interrupts::enable();

	print!("FiddleOS by ");
	let mut writer= text::WRITER.lock();

	writer.display("<TORUS>\n", 0x0D);
	writer.display("Licensed under DUH (latest edition)\n", 0xB0);

	// Let later programs lock onto writer.
	drop(writer);

	prompt();

	loop {
		(APP_SPACE.lock().instruction)()
	}
}



pub fn prompt() {
	shell(['\u{0}'; 128]);

	*driver::keyboard::RAW_KEY.lock() = |key| {
		let mut kb_in = KB_IN.lock();
		let mut writer = WRITER.lock();
		match key {
			KeyCode::ArrowLeft if kb_in.1 != 0 => {
				writer.blink();
				kb_in.1 -= 1;
				writer.cursor -= 1;
				writer.blink();
			}
			KeyCode::ArrowRight if kb_in.1 != 127 => {
				writer.blink();
				kb_in.1 += 1;
				writer.cursor += 1;
				writer.blink();
			}
			_ => {}
		}
		&()
	};

	APP_SPACE.lock().data = &[];

	APP_SPACE.lock().instruction = || {
		// Do nothing.
	}
}


pub fn shell(inp: [char; 128]) -> SizeAny {
		let inp: &[u8] = &inp.map(|x| x as u8);

		let slice_cout = inp.iter().fold(128, |x, y| if *y == 0 {
			x - 1
		} else {
			x
		});

		let inp = &inp[..slice_cout];

		let mut inp = str::from_utf8(inp).unwrap().split(" ");

		// Guaranteed to be safe.
		match inp.next().unwrap() {
			"echo" => {
				print!("{}", match inp.next() {
					Some(input) => input,
					None => ""
				});
				while let Some(text) = inp.next() {
					print!(" {}", text);
				}
				print!("\n")
			},
			"cd" => println!("Hardware storage not supported."),
			"edit" => {
				let mut writer = text::WRITER.lock();
				writer.clear(*text::SCREEN_CLR.lock());
				writer.display("Edit [v.01]\n", 0x5F);

				// Stop deadlock when print!() implicitly
				// happens.
				writer.display("Hello world", 0xCF);

				drop(writer);

				lazy_static! {
					static ref BUF: Mutex<[[char; 128]; 128]> = Mutex::new([
						['\u{0}'; 128]; 128
					]);
					static ref BUF_COUNT: Mutex<usize> = Mutex::new(0);
				}

				*FLUSH_IN.try_lock().unwrap_or_else(|| panic!("Deadlock condition in FLUSH_IN lock. [App: edit]")) = |inp| {
					WRITER.lock().display("Hello world", 0xCF);
					BUF.lock()[*BUF_COUNT.lock()] = inp;
					if *BUF_COUNT.lock() < BUF.lock().len() {
						*BUF_COUNT.lock() += 1;
					} else {
						WRITER.lock().display_overwrite("[Edit mem buffer maxed]", 0xCF);
					}
					&()
				};

				print!("Test2");
			}
			"" => {},
			any => println!("Unknown command: [{}]. Fiddle has no storage support.", any)
		}

		let mut writer = text::WRITER.lock();

		writer.display(" $:", text::PANIC_CLR);
		writer.blink();
		&()
	}