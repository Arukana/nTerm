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
mod spawn;
mod display;

use std::io::Write;
use std::path::PathBuf;
use std::ops::Mul;
use std::env;
use std::thread;
use std::sync::mpsc;

pub use neko::prelude as pty;

use itertools::Itertools;

use gfx_window_glutin as gfxw;
use gfx::Device;
use glutin::GL_CORE;

pub use self::display::Display;
pub use self::err::{NterminalError, Result};

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
    receive: mpsc::Receiver<Display>,
    screen: Display,
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

        let (tx_master, rx_master) = mpsc::sync_channel::<pty::Master>(0);
        let (tx_display, rx_display) = mpsc::sync_channel::<Display>(0);
        thread::spawn(move || spawn::neko(tx_master, tx_display, winszed, font_size));
        let (window, device, mut factory, main_color, _) = {
            let builder = glutin::WindowBuilder::new()
                .with_dimensions(window_size_width, window_size_height)
                .with_title(format!("nTerm"))
                .with_gl(GL_CORE);
            gfxw::init::<gfx::format::Rgba8, gfx::format::Depth>(builder)
        };
        let stream: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer().into();
        let font: PathBuf =
            env::var(SPEC_ROOT).ok()
            .and_then(|repertory: String|
                      Some(PathBuf::from(repertory)))
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join(SPEC_ROOT_DEFAULT))
            .join(SPEC_SUBD_NCF)
            .join(font_name);
        let text = gfx_text::new(factory).with_size(font_size).with_font(font.to_str().expect("font")).unwrap();
        Ok(Nterminal {
            window: window,
            device: device,
            stream: stream,
            text: text,
            main_color: main_color,
            font_size: font_size,
            speudo: rx_master.recv().unwrap(),
            screen: rx_display.recv().unwrap(),
            receive: rx_display,
        })
    }

    pub fn draw(&mut self) -> Option<()> {
        let ref mut text = self.text;
        let font_size: usize = self.font_size as usize;

        if let Ok(screen) = self.receive.try_recv() {
            self.screen = screen;
            if self.screen.is_null() {
                return None ;
            }
        }
        let ref screen = self.screen;
        screen.into_iter()
              .enumerate()
              .foreach(|(y, line):
                        (usize, &[pty::Character])| {
                   line.into_iter()
                       .enumerate()
                       .foreach(|(x, &pty_character)| {
                           let (ref glyph, [fg_r, fg_g, rg_b]) =
                               (pty_character.get_glyph().to_string(),
                                pty_character.get_foreground());
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
        self.stream.clear(&self.main_color, [1.0; 4]);
        text.draw(&mut self.stream, &self.main_color).expect("draw");
        self.stream.flush(&mut self.device);
        self.window.swap_buffers().expect("swap");
        Some(())
    }
}

impl Iterator for Nterminal {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        match self.window.poll_events().next() {
            Some(glutin::Event::Closed) => {
                None
            },
            Some(glutin::Event::ReceivedCharacter(code)) => {
                let (buf, len) = pty::Key::from(code as u32).as_input();
                let _ = self.speudo.write(&buf[..len]);
                Some(())
            },
            Some(glutin::Event::Resized(x, y)) => unsafe {
                let font_size: u32 = self.font_size as u32;
                let (window_size_width, window_size_height): (u32, u32) = (x, y);
                libc::ioctl(libc::STDIN_FILENO, libc::TIOCSWINSZ,
                    &pty::Winszed {
                        ws_col: window_size_width.checked_div(font_size).unwrap_or_default() as u16*2,
                        ws_row: window_size_height.checked_div(font_size).unwrap_or_default() as u16,
                        ws_xpixel: window_size_width as u16,
                        ws_ypixel: window_size_height as u16,
                    }
                );
                Some(())
            },
            None => self.draw(),
            _ => Some(()),
        }
    }
}

impl Drop for Nterminal {
    fn drop(&mut self) {
        self.device.cleanup();
    }
}
