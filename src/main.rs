extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_text;
extern crate glutin;
extern crate itertools;
extern crate neko;

const FONT_PATH: &'static str = "Neko-SourceCodePro-Regular.ttf";
const BACKGROUND_COLOR: [f32; 4] = [0.002, 0.0, 0.0, 0.05];     // RGBA color divide by 255
const FOREGROUND_COLOR: [f32; 4] = [0.65, 0.16, 0.16, 1.0];     // RGBA color divide by 255

use std::ops::Mul;
use std::io::Write;
use std::str;

use neko::prelude as shell;

use itertools::Itertools;

use gfx_window_glutin as gfxw;
use gfx::Device;
use glutin::{GL_CORE, ElementState, MouseButton, VirtualKeyCode};

fn main() {
	let mut shell: shell::Neko = shell::Neko::new(None, None).unwrap();
    let mut with: usize = shell.get_screen().get_window_size().get_col();
	let (window, mut device, mut factory, main_color, _) = {
        let builder = glutin::WindowBuilder::new()
                                            .with_dimensions(840, 480)
                                            .with_title(format!("nTerm"))
                                            .with_gl(GL_CORE);
        gfxw::init::<gfx::format::Rgba8, gfx::format::Depth>(builder)
	};

	let mut stream: gfx::Encoder<_, _> = factory.create_command_buffer().into();
	let mut text = gfx_text::new(factory).with_size(17).with_font(FONT_PATH).unwrap();

	// In render loop:
    'main: loop {
		for event in window.poll_events() {
			match event {
				glutin::Event::Closed => break 'main,
				glutin::Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => {
                    shell.write(b"exit\n");
                    break 'main
                },
				glutin::Event::ReceivedCharacter(code) => unsafe {
                    use std::mem;

                    shell.write(&mem::transmute::<char, [u8; 4]>(code));
                },
				_ => {
                    let shell_event = shell.next().unwrap();

                    if let Some(()) = shell_event.is_signal_resized() {
                        with = shell.get_screen().get_window_size().get_col();
                    }
					if let Some(screen) = shell_event.is_output_screen() {
						// Add some text 10 pixels down and right from the top left screen corner.

                        shell.get_screen()
                             .into_iter()
                             .as_slice()
                             .chunks(with)
                             .enumerate()
                             .foreach(|(y, line)|{
                                  line.iter()
                                      .enumerate()
                                      .foreach(|(x, character)|
                                           text.add(
                                                character.get_unicode().to_string().as_str(),
                                                [10.mul(x as i32), 10.mul(y as i32)],
                                                FOREGROUND_COLOR));
                                       text.add(
                                            "\n",
                                            [10.mul(line.len() as i32), 10.mul(y as i32)],
                                            FOREGROUND_COLOR);                                    
                                      });
						stream.clear(&main_color, BACKGROUND_COLOR);

						// Draw text.
						text.draw(&mut stream, &main_color).unwrap();
						stream.flush(&mut device);
						window.swap_buffers().unwrap();
						device.cleanup();
					}
				},
			}
		}
	}
}

