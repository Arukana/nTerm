extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate gfx_text;
extern crate pty_proc;

const FONT_PATH: &'static str = "Neko-SourceCodePro-Regular.ttf";
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

use pty_proc::prelude as shell;

use gfx_window_glutin as gfxw;
use glutin::{WindowBuilder, GL_CORE, Event, ElementState, MouseButton, VirtualKeyCode};
use gfx::Device;

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

	while let Some(shell_event) = shell.next() {
		for event in window.poll_events() {
			match event {
				Event::Closed => break ,
				Event::MouseInput(ElementState::Pressed, MouseButton::Left) => break ,
				Event::KeyboardInput(ElementState::Pressed, _, _) => break ,
				Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => break ,
				_ => {
					if let Some(screen) = shell_event.is_output_screen() {
						// Add some text 10 pixels down and right from the top left screen corner.
						text.add(
							&format!("{}", unsafe { String::from_utf8_unchecked(screen.into_bytes()) }),
							[10, 10],                                       // Position
							[0.65, 0.16, 0.16, 1.0],                        // Text color
						);
						stream.clear(&main_color, WHITE);

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
