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
extern crate itertools;
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx_text;
extern crate glutin;

/// The module `prelude` is for public.
pub mod prelude;

mod err;

use std::io::Write;
use std::path::PathBuf;
use std::ops::Mul;
use std::mem;
use std::env;

pub use neko::prelude as pty;

use itertools::Itertools;

use gfx_window_glutin as gfxw;
use gfx::Device;
use glutin::{GL_CORE, VirtualKeyCode};

pub use self::err::{NterminalError, Result};

/// The sub-directory font.
const SPEC_SUBD_NCF: &'static str = "etc/fonts";

pub struct Nterminal {
    window: glutin::Window,
    device: gfx_device_gl::Device,
    stream: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    text: gfx_text::Renderer<gfx_device_gl::Resources, gfx_device_gl::Factory>,
    main_color: gfx::handle::RenderTargetView<
        gfx_device_gl::Resources,
        (gfx::format::R8_G8_B8_A8, gfx::format::Unorm)
    >,
    /// The shell interface.
    shell: pty::Neko,
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
        let shell: pty::Neko = pty::Neko::new(None, None, None, Some(winszed)).expect("neko");
        let (window, device, mut factory, main_color, _) = {
            let builder = glutin::WindowBuilder::new()
                .with_dimensions(window_size_width, window_size_height)
                .with_title(format!("nTerm"))
                .with_gl(GL_CORE);
            gfxw::init::<gfx::format::Rgba8, gfx::format::Depth>(builder)
        };
        let stream: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> = factory.create_command_buffer().into();
        let font: PathBuf = PathBuf::from(
                env::var(neko::SPEC_ROOT).ok().unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").to_string())
            )
            .join(SPEC_SUBD_NCF)
            .join(font_name);
        let text = gfx_text::new(factory).with_size(font_size).with_font(font.to_str().expect("font")).unwrap();
    
        Ok(Nterminal {
            window: window,
            device: device,
            stream: stream,
            text: text,
            main_color: main_color,
            shell: shell,
            font_size: font_size,
        })
    }


    pub fn draw(&mut self) {
        let font_size: usize = self.font_size as usize;
        let shell_event = self.shell.next().expect("next");
        let window_size: &pty::Winszed = self.shell.get_window_size();
        let width: usize = window_size.get_xpixel().checked_div(font_size as u32).unwrap_or_default() as usize*2;
        let ref mut text = self.text;

        if let Some(()) = shell_event.is_output_screen() {
            self.shell.get_screen()
                .into_iter()
                .as_slice()
                .chunks(width)
                .enumerate()
                .foreach(|(y, line)| {
                         line.iter().enumerate().all(|(x, &character)| {
                             let [fg_r, fg_g, rg_b] = character.get_foreground();
                             text.add(character.get_glyph().to_string().as_str(),
                                      [font_size.mul(&x) as i32/2,
                                       font_size.mul(&y) as i32],
                                      [fg_r as f32, fg_g as f32, rg_b as f32, 1.0]
                             );
                             true
                         });
                });
            self.stream.clear(&self.main_color, [1.0; 4]);
            text.draw(&mut self.stream, &self.main_color).expect("draw");
            self.stream.flush(&mut self.device);
            self.window.swap_buffers().expect("swap");
        }
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
                self.shell.write(b"exit\n").expect("exit");
                Some(())
            },
            Some(glutin::Event::ReceivedCharacter(code)) => unsafe {
                self.shell.write(&mem::transmute::<char, [u8; 4]>(code)).expect("transmutation");
                Some(())
            },
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
