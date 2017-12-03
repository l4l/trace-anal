#[macro_use]
mod parsing;
mod cfg;
pub use cfg::Cfg;
mod trace;
pub use trace::*;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let usage = format!("Use {} <json-file>", env::args().nth(0).unwrap());
    let file = env::args().nth(1).expect(&usage);

    let mut content = String::new();
    File::open(&file)
        .expect(&format!("Can't open {}", &file))
        .read_to_string(&mut content)
        .expect("Something happend during file reading");
    let cfg = Cfg::linear(Bb::new(parsing::parse_trace(&content)));
    println!("{:?}", cfg);
}
