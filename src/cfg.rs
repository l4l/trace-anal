use std::collections::HashMap;
use trace::Bb;

#[derive(Debug)]
pub struct Node {
    pub was: bool,
    pub bb: Bb,
}

#[derive(Debug)]
pub struct Cfg {
    pub verts: Vec<Node>,
    pub edges: HashMap<u64, u64>,
}

impl Cfg {
    pub fn linear(v: Vec<Bb>) -> Cfg {
        Cfg {
            verts: v.into_iter().map(|x| Node { was: false, bb: x }).collect(),
            edges: HashMap::new(),
        }
    }
}
