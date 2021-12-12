use lazy_static::lazy_static;

use crate::{ 
	print, 
	println, 
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

lazy_static! {
    pub static ref KB_IN:Mutex<([char; 128], usize)> = Mutex::new((['\u{0}'; 128], 0));
}



pub extern "x86-interrupt" fn keyboard_handler(
		stack_frame: InterruptStackFrame
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
					let mut kb_in = KB_IN.lock();
            match key {
                DecodedKey::Unicode(character) => match character {
                    '\u{0008}' => { 
												text::WRITER.lock().blink();
												if kb_in.1 != 0 {
													kb_in.1 -= 1;
												}
                        text::WRITER.lock().clear_last();
												text::WRITER.lock().blink();
                    },
                    '\n' => {
												text::WRITER.lock().blink();
                        println!();
                        crate::prompt(kb_in.0);
                        kb_in.0 = ['\u{0}'; 128];
												kb_in.1 = 0

                    }
                    character=> { 
                        let cursor = kb_in.1;
                        kb_in.0[cursor] = character;
												if cursor+1 > 127 {
													text::WRITER.lock().display_overwrite("[Input buffer length maxed. Del or press confirm.]", 0xCF)
												} else {
													print!("{}", character);
													text::WRITER.lock().blink();
													kb_in.1 += 1;
												}
                        drop(kb_in)
                    }
                },
                DecodedKey::RawKey(key) => match key {
										KeyCode::ArrowLeft => { if kb_in.1 != 0 {
											kb_in.1 -= 1;
											let mut writer = text::WRITER.lock();
											writer.blink();
											writer.cursor -= 1;
											writer.blink();
										} }
										KeyCode::ArrowRight => { if kb_in.1 < 127 {
											kb_in.1 += 1;
											let mut writer = text::WRITER.lock();
											writer.blink();
											writer.cursor += 1;
											writer.blink();
										} }
                    any => print!("{:?}", any)
                },
            }
        }
    }



	unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
	}
}
