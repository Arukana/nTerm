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
extern crate piston_window;
extern crate graphics;
extern crate gfx_device_gl;
extern crate gfx_graphics;

/// The module `prelude` is for public.
pub mod prelude;

mod err;

use std::mem;
use std::ops::Mul;
use std::io::Write;
use std::path::Path;

pub use neko::prelude as pty;
pub use piston_window::*;
use itertools::Itertools;

pub use self::err::{NterminalError, Result};

/// The sub-directory font.
const SPEC_SUBD_NCF: &'static str = "etc/fonts";

pub struct Nterminal {
    pty: pty::Neko,
    window: PistonWindow,
    glyph: Glyphs,
    window_size: [u32; 2],
    font_size: u32,
}

impl Nterminal {
    pub fn new (
        font_name: &str,
        font_size: u32,
        [window_size_width, window_size_height]: [u32; 2],
    ) -> Result<Self> {
        let winszed: pty::Winszed = pty::Winszed {
            ws_col: window_size_width.checked_div(font_size).unwrap_or_default() as u16,
            ws_row: window_size_height.checked_div(font_size).unwrap_or_default() as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let window: PistonWindow = 
            try!(WindowSettings::new("nTerm", [window_size_width, window_size_height])
                                .exit_on_esc(false)
                                .build());
        Ok(Nterminal {
            pty: try!(pty::Neko::new(None, None, None, Some(winszed))),
            glyph: try!(Glyphs::new(Path::new(env!("CARGO_MANIFEST_DIR"))
                                         .join(SPEC_SUBD_NCF)
                                         .join(font_name),
                                    window.factory.clone())),
            window: window,
            window_size: [window_size_width, window_size_height],
            font_size: font_size,
        })
    }

    pub fn draw(&mut self, event: &piston_window::Event) {
        let font_size: usize = self.font_size as usize;
        let width: usize = self.window_size[0].checked_div(font_size as u32).unwrap_or_default() as usize;
        let ref mut glyph = self.glyph;
        let ref mut neko = self.pty.get_screen();

        self.window.draw_2d(event, |c, g| {
            clear([0.0, 0.0, 0.0, 1.0], g);
             neko
                .into_iter()
                .as_slice()
                .chunks(width)
                .enumerate()
                .foreach(|(y, line)| {
                    line.iter().enumerate().all(|(x, &character)| {
                        let transform = c.transform.trans(font_size.mul(&x) as f64,
                                                          font_size.mul(&y) as f64);
                        text::Text::new_color([0.0, 1.0, 0.0, 1.0], font_size as u32)
                             .draw(
                                  character.get_glyph().to_string().as_str(),
                                  glyph,
                                  &c.draw_state,
                                  transform, g
                             );
                        true
                    });
                });
        });
    }
}

impl Iterator for Nterminal {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        self.window.next().and_then(|event: piston_window::Event| {
            match event {
                Event::Render(_) => {
                    self.pty.next().and_then(|pty_event| {
                            self.draw(&event);
 /*                       if let Some(()) = pty_event.is_output_screen() {
                        }*/
                        Some(())
                    })
                },
                Event::Input(Input::Press(Button::Keyboard(key))) => unsafe {
                    self.pty.write(&mem::transmute::<u32, [u8; 4]>(key as u32)).ok().and_then(|_| Some(()))
                },
                Event::Input(Input::Press(Button::Mouse(mouse))) => {
                    println!("mouse: {:?}", mouse);
                    None
                },
                Event::Input(Input::Move(Motion::MouseCursor(_, _))) => {
                    Some(())
                },
                Event::Input(Input::Text(paste)) => {
                    self.pty.write(paste.as_bytes()).ok().and_then(|_| Some(()))
                },
                Event::Input(Input::Resize(x, y)) => {
                    self.window_size = [x, y];
                    self.pty.set_window_size_with(
                        &pty::Winszed {
                            ws_col: x.checked_div(self.font_size).unwrap_or_default() as u16,
                            ws_row: y.checked_div(self.font_size).unwrap_or_default() as u16,
                            ws_xpixel: 0,
                            ws_ypixel: 0,
                        }
                    );
                    Some(())
                },
                Event::Input(Input::Close) => {
                    None
                },
                _ => Some(()),
            }
        })
    }
}
