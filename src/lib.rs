#![feature(slice_patterns)]

#![crate_type= "lib"]
#![cfg_attr(feature = "nightly", feature(plugin))]

#![feature(plugin)]
#![cfg_attr(feature = "clippy", plugin(clippy(conf_file="clippy.toml")))]

#![cfg_attr(feature = "lints", plugin(clippy))]
#![cfg_attr(feature = "lints", deny(warnings))]
#![cfg_attr(not(any(feature = "lints", feature = "nightly")), deny())]
#![deny(
//    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
//    unused_qualifications
)]

extern crate neko;
extern crate libc;
extern crate itertools;
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx_text;
extern crate glutin;

/// The module `prelude` is for public.
pub mod prelude;

mod err;
mod state;

use std::io::Write;
use std::path::PathBuf;
use std::ops::Mul;
use std::mem;
use std::env;
use std::thread;
use std::sync::mpsc;

pub use neko::prelude as pty;

use itertools::Itertools;

use gfx_window_glutin as gfxw;
use gfx::Device;
use glutin::{GL_CORE, VirtualKeyCode};

pub use self::err::{NterminalError, Result};
pub use self::state::{NterminalState, Display};

/// The sub-directory font.
const SPEC_SUBD_NCF: &'static str = "fonts";
/// The first directory.
pub const SPEC_ROOT: &'static str = "NEKO_PATH";
pub const SPEC_ROOT_DEFAULT: &'static str = "assets";

pub struct Nterminal {
    window: glutin::Window,
    device: gfx_device_gl::Device,
    stream: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    text: gfx_text::Renderer<gfx_device_gl::Resources, gfx_device_gl::Factory>,
    main_color: gfx::handle::RenderTargetView<
        gfx_device_gl::Resources,
        (gfx::format::R8_G8_B8_A8, gfx::format::Unorm)
    >,
    speudo: pty::Master,
    receive: mpsc::Receiver<NterminalState>,
    state: NterminalState,
    /// The font size.
    font_size: u8,
}

impl Nterminal {

    pub fn new (
        font_name: &str,
        font_size: u8,
        [window_size_width, window_size_height]: [u32; 2],
    ) -> Result<Self> {
        let winszed: pty::Winszed = pty::Winszed {
            ws_col: window_size_width.checked_div(font_size as u32).unwrap_or_default() as u16*2,
            ws_row: window_size_height.checked_div(font_size as u32).unwrap_or_default() as u16,
            ws_xpixel: window_size_width as u16,
            ws_ypixel: window_size_height as u16,
        };

        let (tx, rx) = mpsc::sync_channel::<NterminalState>(0);
        thread::spawn(move || {
            let mut shell: pty::Neko =
                pty::Neko::new(None, None, None, Some(winszed)).expect("neko");
            /// Send a copy of master interface to write on the process' child.
            let _ = tx.send(NterminalState::Master(*shell.get_speudo()));
            let window_size: pty::Winszed = *shell.get_window_size();
            let width: usize = window_size.get_xpixel().checked_div(font_size as u32).unwrap_or_default() as usize*2;

            while let Some(event) = shell.next() {
                if event.is_output_screen().is_some() {
                    let (pty_screen, screen): (&pty::PtyDisplay, &pty::Display) =
                        shell.get_screen();
                    let _ = tx.send(NterminalState::Display(Display::from((width, pty_screen.into_iter().cloned().into_iter()
                      .zip(screen.into_iter())
                      .collect::<Vec<(pty::Character, pty::Character)>>())))
                    );
                }
            }
        });
        let (window, device, mut factory, main_color, _) = {
            let builder = glutin::WindowBuilder::new()
                .with_dimensions(window_size_width, window_size_height)
                .with_title(format!("nTerm"))
                .with_gl(GL_CORE);
            gfxw::init::<gfx::format::Rgba8, gfx::format::Depth>(builder)
        };
        let stream: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> = factory.create_command_buffer().into();
        let font: PathBuf =
            env::var(SPEC_ROOT).ok()
            .and_then(|repertory: String|
                      Some(PathBuf::from(repertory)))
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join(SPEC_ROOT_DEFAULT))
            .join(SPEC_SUBD_NCF)
            .join(font_name);
        let text = gfx_text::new(factory).with_size(font_size).with_font(font.to_str().expect("font")).unwrap();
        
        if let Ok(NterminalState::Master(speudo)) = rx.recv() {
            Ok(Nterminal {
                window: window,
                device: device,
                stream: stream,
                text: text,
                main_color: main_color,
                font_size: font_size,
                speudo: speudo,
                state: NterminalState::Master(speudo),
                receive: rx,
            })
        } else {
            unimplemented!()
        }
    }


    pub fn draw(&mut self) {
        let ref mut text = self.text;
        let font_size: usize = self.font_size as usize;

        if let Ok(state) = self.receive.try_recv() {
            self.state = state;
        }
        if let NterminalState::Display(ref screen) = self.state {
            screen.into_iter()
                  .enumerate()
                  .foreach(|(y, line):
                            (usize, &[(pty::Character, pty::Character)])| {
                       line.into_iter()
                           .enumerate()
                           .foreach(|(x, &(pty_character, character))| {
                               let (ref glyph, [fg_r, fg_g, rg_b]) =
                                   if pty_character.is_space() {
                                       (character.get_glyph().to_string(),
                                        character.get_foreground())
                                   } else {
                                       (pty_character.get_glyph().to_string(),
                                        pty_character.get_foreground())
                                   };
                               text.add(glyph,
                                        [font_size.mul(&x) as i32/2,
                                         font_size.mul(&y) as i32],
                                        [fg_r as f32,
                                         fg_g as f32,
                                         rg_b as f32,
                                         1.0]
                               );
                           });
                  });
        }
        self.stream.clear(&self.main_color, [1.0; 4]);
        text.draw(&mut self.stream, &self.main_color).expect("draw");
        self.stream.flush(&mut self.device);
        self.window.swap_buffers().expect("swap");
    }
}

impl Iterator for Nterminal {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        match self.window.poll_events().next() {
            Some(glutin::Event::Closed) => {
                None
            },
            Some(glutin::Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape))) => {
                self.speudo.write(b"exit\n").expect("exit");
                Some(())
            },
            Some(glutin::Event::ReceivedCharacter(code)) => unsafe {
                self.speudo.write(&mem::transmute::<char, [u8; 4]>(code)).expect("transmutation");
                Some(())
            },
            /*Some(glutin::Event::Resized(x, y)) => {
                let font_size: u32 = self.font_size as u32;
                let (window_size_width, window_size_height): (u32, u32) = (x, y);
                self.shell.set_window_size_with(
                    &pty::Winszed {
                        ws_col: window_size_width.checked_div(font_size).unwrap_or_default() as u16*2,
                        ws_row: window_size_height.checked_div(font_size).unwrap_or_default() as u16,
                        ws_xpixel: window_size_width as u16,
                        ws_ypixel: window_size_height as u16,
                    }
                );
                Some(())
            },*/
            None => {
                self.draw();
                Some(())
            },
            _ => Some(()),
        }
    }
}

impl Drop for Nterminal {
    fn drop(&mut self) {
        self.device.cleanup();
    }
}
