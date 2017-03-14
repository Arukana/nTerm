use ::pty;
use ::pty::Parent;
use std::sync::mpsc;

use super::Display;

pub fn neko(
    tx_master: mpsc::SyncSender<pty::Master>,
    tx_display: mpsc::SyncSender<Display>,
    winszed: pty::Winszed,
    font_size: u8,
) {
    let mut shell: pty::Shell =
        pty::Shell::new(None, None, None, Some(winszed)).expect("neko");
    /// Send a copy of master interface to write on the process' child.
    let _ = tx_master.send(*shell.get_speudo());
    drop(tx_master);

    let window_size: pty::Winszed = *shell.get_window_size();
    let mut width: usize = window_size.get_xpixel().checked_div(font_size as u32).unwrap_or_default() as usize*2;

    while let Some(event) = <pty::Shell as Iterator>::next(&mut shell) {
        if let Some(window_size) = event.is_resized() {
            width = window_size.get_xpixel().checked_div(font_size as u32).unwrap_or_default() as usize*2;
        }
        if event.is_output_screen().is_some() {
            let pty_screen: &pty::PtyDisplay = shell.get_screen();
            let _ = tx_display.send(
                Display::from((width, pty_screen.into_iter().cloned().into_iter()
                  .collect::<Vec<pty::Character>>()
                ))
            );
        }
    }
    let _ = tx_display.send(Display::default());
}
