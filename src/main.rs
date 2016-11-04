extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate gfx_text;
extern crate pty_proc;

const FONT_PATH: &'static str = "Neko-SourceCodePro-Regular.ttf";
const PIXEL: i32 = 1;   // Screen coordinates in pixels 
const BACKGROUND_COLOR: [f32; 4] = [0.002, 0.0, 0.0, 0.05];     // RGBA color divide by 255
const FOREGROUND_COLOR: [f32; 4] = [0.65, 0.16, 0.16, 1.0];     // RGBA color divide by 255

use pty_proc::prelude as shell;

use gfx_window_glutin as gfxw;
use glutin::{WindowBuilder, GL_CORE, Event, ElementState, MouseButton, VirtualKeyCode};
use gfx::Device;

use std::io::Write;
use std::str;

fn main() {
	let mut shell: shell::Shell = shell::Shell::from_mode(None, None, None, shell::Mode::Character).unwrap();
	let (window, mut device, mut factory, main_color, _) = {
        let builder = WindowBuilder::new()
						            .with_dimensions(840, 480)
						            .with_title(format!("nTerm"))
							        .with_gl(GL_CORE);
        gfxw::init::<gfx::format::Rgba8, gfx::format::Depth>(builder)
	};

	let mut stream: gfx::Encoder<_, _> = factory.create_command_buffer().into();
	let mut text = gfx_text::new(factory).with_size(17).with_font(FONT_PATH).unwrap();

	// In render loop:

    shell.write(b"echo nya \n sssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss \n\n");
	while let Some(shell_event) = shell.next() {
		for event in window.poll_events() {
			match event {
				Event::Closed => break ,
				Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => break ,
				_ => {
					if let Some(screen) = shell_event.is_output_screen() {
						// Add some text 10 pixels down and right from the top left screen corner.
                        let display: String = unsafe {
                            String::from_utf8_unchecked(screen.into_bytes())
                        };

                        let display: &str = display.as_str();
                        (0..{display.len()/80}).all(|index| unsafe {
                            let begin = index*80;
                            let end = begin+80;
                            let line: &str = display.slice_unchecked(begin, end);
                            text.add(
                                line,
                                [10 * PIXEL, 12 * PIXEL * index as i32],    // Position
                                FOREGROUND_COLOR,                    // Text color
                            );
                            true
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

