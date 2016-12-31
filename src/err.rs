use ::pty;
use ::piston_window;

use std::error::Error;
use std::fmt;

pub type Result<T> = ::std::result::Result<T, NterminalError>;

/// The enum `NterminalError` defines the possible errors
/// from constructor Nterminal.
#[derive(Debug)]
pub enum NterminalError {
    WindowSettings(String),
    Neko(pty::NekoError),
    Graphic(pty::GraphicError),
    Glyph(piston_window::GlyphError),
}

impl fmt::Display for NterminalError {
    /// The function `fmt` formats the value using
    /// the given formatter.
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl Error for NterminalError {
    /// The function `description` returns a short description of
    /// the error.
    fn description(&self) -> &str {
        match *self {
            NterminalError::WindowSettings(ref why) => why,
            NterminalError::Neko(_) => "The neko interface has occured an error.",
            NterminalError::Graphic(_) => "The Graphic has occured an error.",
            NterminalError::Glyph(_) => "The piston_window::Glyph has occured an error."
        }
    }

    /// The function `cause` returns the lower-level cause of
    /// this error if any.
    fn cause(&self) -> Option<&Error> {
        match *self {
            NterminalError::Neko(ref why) => why.cause(),
            _ => None,
        }
    }
}

impl From<String> for NterminalError {
    fn from(err: String) -> NterminalError {
        NterminalError::WindowSettings(err)
    }
}

impl From<pty::NekoError> for NterminalError {
    fn from(err: pty::NekoError) -> NterminalError {
        NterminalError::Neko(err)
    }
}

impl From<pty::GraphicError> for NterminalError {
    fn from(err: pty::GraphicError) -> NterminalError {
        NterminalError::Graphic(err)
    }
}

impl From<piston_window::GlyphError> for NterminalError {
    fn from(err: piston_window::GlyphError) -> NterminalError {
        NterminalError::Glyph(err)
    }
}
