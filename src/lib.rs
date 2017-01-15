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
mod key;

use std::ops::Mul;
use std::io::Write;
use std::path::Path;
use std::str;

pub use neko::prelude as pty;
pub use piston_window::*;
use itertools::Itertools;

pub use self::err::{NterminalError, Result};

/// The sub-directory font.
const SPEC_SUBD_NCF: &'static str = "assets/fonts";

pub struct Nterminal {
    window: PistonWindow,
    glyph: Glyphs,
    /// The shell interface.
    shell: pty::Neko,
    /// The font size.
    size: u8,
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
        println!("{:?}", winszed);
        let window: PistonWindow = 
            try!(WindowSettings::new("nTerm", [winszed.get_xpixel(), winszed.get_ypixel()])
                                .exit_on_esc(false)
                                .build());

        Ok(Nterminal {
            shell: try!(pty::Neko::new(None, None, None, Some(winszed))),
            size: font_size,
            glyph: try!(Glyphs::new(Path::new(env!("CARGO_MANIFEST_DIR"))
                                         .join(SPEC_SUBD_NCF)
                                         .join(font_name),
                                    window.factory.clone())),
            window: window,
        })
    }

    pub fn draw(&mut self, event: &piston_window::Event) {
        self.shell.next();
        let font_size: usize = self.size as usize;
        let window_size: &pty::Winszed = self.shell.get_window_size();
        let width: usize = window_size.get_xpixel().checked_div(font_size as u32).unwrap_or_default() as usize*2;
        let ref mut glyph = self.glyph;
        let (ref pty_screen, ref screen): (&pty::PtyDisplay, &pty::Display) = self.shell.get_screen();

        self.window.draw_2d(event, |c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
                pty_screen.into_iter()
                    .zip(screen.into_iter())
                    .collect::<Vec<(&pty::Character, pty::Character)>>()
                    .as_slice()
                    .chunks(width)
                    .enumerate()
                    .foreach(|(y, line): (usize, &[(&pty::Character, pty::Character)])| {
                             line.into_iter().enumerate().foreach(|(x, &(&pty_character, character))| {
                                 let (ref character, [fg_r, fg_g, rg_b]) = if character.get_glyph().eq(&'\0') {
                                     (pty_character.get_glyph().to_string(), pty_character.get_foreground())
                                 } else {
                                    (character.get_glyph().to_string(), character.get_foreground())
                                 };
                                 let transform = c.transform.trans((font_size.mul(&x) as f64/2.0), (font_size.mul(&y) + font_size) as f64);
                                 text::Text::new_color([fg_r as f32, fg_g as f32, rg_b as f32, 1.0], font_size as u32)
                                      .draw(
                                           character,
                                           glyph,
                                           &c.draw_state,
                                           transform, g
                                      );
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
                    self.draw(&event);
                    Some(())
                },
                Event::Input(Input::Press(Button::Keyboard(key))) => {
                    let buf: [u8; 8] = key::Key::from(key).as_slice();

                    let _ = self.shell.write(&buf.split_at(buf.iter().position(|c| b'\0'.eq(c)).unwrap_or_default()).0);
                    Some(())
                },
                Event::Input(Input::Press(Button::Mouse(mouse))) => {
                    Some(())
                },
                Event::Input(Input::Move(Motion::MouseCursor(_, _))) => {
                    Some(())
                },
                Event::Input(Input::Text(paste)) => {
//                    self.shell.write(paste.as_bytes()).ok().and_then(|_| Some(()))
                    Some(())
                },
                Event::Input(Input::Resize(x, y)) => {
                    let font_size: u32 = self.size as u32;
                    let (window_size_width, window_size_height): (u32, u32) =
                        (x, y);
                    self.shell.set_window_size_with(
                        &pty::Winszed {
                            ws_col: window_size_width.checked_div(font_size).unwrap_or_default() as u16*2,
                            ws_row: window_size_height.checked_div(font_size).unwrap_or_default() as u16,
                            ws_xpixel: window_size_width as u16,
                            ws_ypixel: window_size_height as u16,
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
