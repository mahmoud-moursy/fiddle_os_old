use lazy_static::lazy_static;

use crate::{ 
	interrupts::{ PICS, InterruptIndex },
	InterruptStackFrame,
};

use crate::text;

use pc_keyboard::{
	ScancodeSet1,
	Keyboard,
	layouts,
	HandleControl,
	DecodedKey, KeyCode,
};

use spin::Mutex;
use core::any::Any;

pub type SizeAny = &'static (dyn Any + Send + Sync);

lazy_static! {
    pub static ref KB_IN:Mutex<([char; 128], usize)> = Mutex::new((['\u{0}'; 128], 0));
		pub static ref FLUSH_KEY: Mutex<char> = Mutex::new('\n');
		pub static ref FLUSH_IN: Mutex<fn([char; 128]) -> SizeAny> = Mutex::new(crate::shell);
		pub static ref ON_KEY: Mutex<fn(char) -> SizeAny> = Mutex::new(on_key);
		pub static ref RAW_KEY: Mutex<fn(KeyCode) -> SizeAny> = Mutex::new(on_raw);
}

use crate::print;

pub extern "x86-interrupt" fn keyboard_handler(
		_stack_frame: InterruptStackFrame
) {
	use x86_64::instructions::port::Port;

	let mut port = Port::new(0x60);
	let scancode: u8 = unsafe { port.read() };


	lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

	let mut keyboard = KEYBOARD.lock();

	if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
									let mut kb_in = KB_IN.lock();
									ON_KEY.lock()(character);
									match character {
										character if character == *FLUSH_KEY.lock() => {
												let flush_fn = match FLUSH_IN.try_lock() { 
													Some(func) => func,
													None => { panic!("Deadlock condition in ON_KEY call. (Not user/developer error, OS error)") }
												};
												flush_fn(kb_in.0);
												print!("\nUnlocked!");
                        kb_in.0 = ['\u{0}'; 128];
												kb_in.1 = 0;
                    }
                    '\u{0008}' => { 
												if kb_in.1 != 0 {
													kb_in.1 -= 1;
												}
												let idx = kb_in.1;
												kb_in.0[idx] = '\u{0}';
                    },
                    character=> { 
                        let cursor = kb_in.1;
                        kb_in.0[cursor] = character;
												if cursor+1 > 127 {
													text::WRITER.lock().display_overwrite("[Input buffer length maxed.]", 0xCF)
												} else {
													kb_in.1 += 1;
												}
                        drop(kb_in)
                    }
                }},
                DecodedKey::RawKey(key) => {RAW_KEY.lock()(key);},
            }
        }
    }



	unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
	}
}

pub fn flush_in(_: [char; 128]) -> SizeAny {
	crate::text::WRITER.lock().display_overwrite("[Cannot flush input]", 0xCF);
	&()
}

pub fn on_key(inp: char) -> SizeAny {
	let mut writer = text::WRITER.lock();
	match inp {
		'\u{008}' => { writer.blink(); writer.clear_last(); writer.blink(); },
		any => { writer.blink(); writer.display(unsafe { core::str::from_utf8_unchecked(&[any as u8]) }, 0x0F); writer.blink(); }
	}
	&()
}

pub fn on_raw(key: KeyCode) -> SizeAny {
	&()
}