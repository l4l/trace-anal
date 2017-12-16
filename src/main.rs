extern crate itertools;

#[macro_use]
mod parsing;
mod cfg;
use cfg::Cfg;
mod trace;
use trace::Bb;
mod graph;
mod base;

use std::env;
use std::fs::File;
use std::io::{Read, stdout};
use std::fmt;

impl fmt::Display for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut prnt = format!("Cfg ###\nVetices: {{");
        for (&k, _) in self.verts.iter() {
            prnt.push_str(&format!(" addr: {}\n", k));
        }
        prnt.push_str(&format!("}}\nEdges: {{"));
        for (&l, ref r) in self.edges.iter() {
            prnt.push_str(&format!(" l: {:16}, r: {:?}\n", l, r))
        }
        prnt.push_str(&format!("}}\n###"));
        f.write_str(&prnt)
    }
}

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
    let cfg = Cfg::from_blocks(Bb::new(parsing::parse_trace(&content)));
    println!("{}", cfg);

    if let Some(fname) = env::args().nth(2) {
        cfg.render_to(&mut File::create(fname).unwrap());
    } else {
        cfg.render_to(&mut stdout());
    }
}
