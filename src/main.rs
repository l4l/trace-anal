extern crate itertools;

#[macro_use]
mod parsing;
mod cfg;
pub use cfg::Cfg;
mod trace;
pub use trace::*;
mod graph;

use std::env;
use std::fs::File;
use std::io::{Read, stdout};

fn main() {
    let usage = format!(
        "Use {} <json-file> [<output-dotfile>]",
        env::args().nth(0).unwrap()
    );
    let file = env::args().nth(1).expect(&usage);

    let mut content = String::new();
    File::open(&file)
        .expect(&format!("Can't open {}", &file))
        .read_to_string(&mut content)
        .expect("Something happend during file reading");
    let cfg = Cfg::linear(Bb::new(parsing::parse_trace(&content)));

    if let Some(fname) = env::args().nth(2) {
        cfg.render_to(&mut File::create(fname).unwrap());
    } else {
        cfg.render_to(&mut stdout());
    }
}
