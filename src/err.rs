use ::pty;

use std::error::Error;
use std::fmt;

pub type Result<T> = ::std::result::Result<T, NterminalError>;

/// The enum `NterminalError` defines the possible errors
/// from constructor Nterminal.
#[derive(Debug)]
pub enum NterminalError {
    Neko(pty::NekoError),
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
            NterminalError::Neko(_) => "The neko interface has occured an error.",
        }
    }

    /// The function `cause` returns the lower-level cause of
    /// this error if any.
    fn cause(&self) -> Option<&Error> {
        match *self {
            NterminalError::Neko(ref why) => why.cause(),
//            _ => None,
        }
    }
}

impl From<pty::NekoError> for NterminalError {
    fn from(err: pty::NekoError) -> NterminalError {
        NterminalError::Neko(err)
    }
}

