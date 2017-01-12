#![feature(slice_patterns)]

#[macro_use]
extern crate clap;
extern crate nterm;

use nterm::prelude::*;

/// Default* const arguments defined by CLI.

const DEFAULT_FONT_NAME: &'static str = "Neko-SourceCodePro-Regular.ttf";
const DEFAULT_FONT_SIZE: u32 = 15;
const DEFAULT_WINDOW_SIZE_WIDTH: u32 = DEFAULT_FONT_SIZE * 80;
const DEFAULT_WINDOW_SIZE_HEIGHT: u32 = DEFAULT_FONT_SIZE * 24;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let options = clap::App::from_yaml(yaml).get_matches();
    
    let mut nterm: Nterminal = Nterminal::new(
        options.value_of("font-name").unwrap_or_else(|| DEFAULT_FONT_NAME),
        options.value_of("font-size").and_then(|size| size.parse::<u8>().ok()).unwrap_or_else(|| DEFAULT_FONT_SIZE as u8),
        [options.value_of("window-size-width").and_then(|width| width.parse::<u32>().ok()).unwrap_or_else(|| DEFAULT_WINDOW_SIZE_WIDTH),
         options.value_of("window-size-height").and_then(|height| height.parse::<u32>().ok()).unwrap_or_else(|| DEFAULT_WINDOW_SIZE_HEIGHT)],
    ).unwrap();

    loop {
        if nterm.next().is_none() {
            break ;
        }
    }
}
