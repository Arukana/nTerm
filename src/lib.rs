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
extern crate piston_window;
extern crate graphics;
extern crate gfx_device_gl;
extern crate gfx_graphics;

/// The module `prelude` is for public.
pub mod prelude;

mod err;
mod key;
mod spawn;
mod display;

use std::ops::Mul;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::sync::mpsc;
use std::str;
use std::mem;

pub use neko::prelude as pty;
pub use piston_window::*;
use itertools::Itertools;

pub use self::display::Display;
pub use self::err::{NterminalError, Result};

/// The sub-directory font.
const SPEC_SUBD_NCF: &'static str = "assets/fonts";

pub struct Nterminal {
    window: PistonWindow,
    text: Glyphs,
    speudo: pty::Master,
    receive: mpsc::Receiver<Display>,
    screen: Display,
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
        let window: PistonWindow = 
            try!(WindowSettings::new("nTerm", [winszed.get_xpixel(), winszed.get_ypixel()])
                                .exit_on_esc(false)
                                .build());

        let (tx_master, rx_master) = mpsc::sync_channel::<pty::Master>(0);
        let (tx_display, rx_display) = mpsc::sync_channel::<Display>(0);
        thread::spawn(move || spawn::neko(tx_master, tx_display, winszed, font_size));
        Ok(Nterminal {
            size: font_size,
            text: try!(Glyphs::new(Path::new(env!("CARGO_MANIFEST_DIR"))
                                         .join(SPEC_SUBD_NCF)
                                         .join(font_name),
                                    window.factory.clone())),
            window: window,

            speudo: rx_master.recv().unwrap(),
            screen: rx_display.recv().unwrap(),
            receive: rx_display,
        })
    }

    pub fn draw(&mut self, event: &piston_window::Event) -> Option<()> {
        let ref mut text = self.text;
        let font_size: usize = self.size as usize;


        if let Ok(screen) = self.receive.try_recv() {
            self.screen = screen;
            if self.screen.is_null() {
                return None;
            }
        }
        let ref screen = self.screen;
        self.window.draw_2d(event, |c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            screen.into_iter()
                  .enumerate()
                  .foreach(|(y, line): (usize, &[(pty::Character, pty::Character)])| {
                       line.into_iter()
                           .enumerate()
                          .foreach(|(x, &(pty_character, character))| {
                                 let (ref character, [fg_r, fg_g, rg_b]) = if character.is_null() {
                                     (pty_character.get_glyph().to_string(), pty_character.get_foreground())
                                 } else {
                                    (character.get_glyph().to_string(), character.get_foreground())
                                 };
                                 let transform = c.transform.trans((font_size.mul(&x) as f64/2.0), (font_size.mul(&y) + font_size) as f64);
                                 text::Text::new_color([fg_r as f32, fg_g as f32, rg_b as f32, 1.0], font_size as u32)
                                      .draw(
                                           character,
                                           text,
                                           &c.draw_state,
                                           transform, g
                                      );
                             });
                });
        });
        Some(())
    }
}

impl Iterator for Nterminal {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        self.window.next().and_then(|event: piston_window::Event| {
            match event {
                Event::Input(Input::Close) => None,
                Event::Input(Input::Press(Button::Keyboard(code))) => unsafe {
                    self.speudo.write(&mem::transmute::<piston_window::Key, [u8; 4]>(code)).expect("transmutation");
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
                Event::Input(Input::Resize(x, y)) => unsafe {
                    let font_size: u32 = self.size as u32;
                    let (window_size_width, window_size_height): (u32, u32) =
                        (x, y);
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
                Event::Render(_) => self.draw(&event),
                _ => Some(()),
            }
        })
    }
}
