use ::pty;

mod display;

pub use self::display::Display;

#[derive(Debug, Clone)]
pub enum NterminalState {
    Master(pty::Master),
    Display(Display),
}
