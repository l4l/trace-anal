use itertools::Itertools;
use std::collections::{HashMap, BTreeMap};
use trace::Bb;

#[derive(Debug)]
pub struct Node {
    pub was: bool,
    pub bb: Bb,
}

impl Node {
    fn new(b: Bb) -> Node {
        Node { was: false, bb: b }
    }
}

#[derive(Debug)]
pub struct Cfg {
    pub verts: BTreeMap<usize, Node>,
    pub edges: HashMap<usize, usize>,
}

impl Cfg {
    pub fn linear(v: Vec<Bb>) -> Cfg {
        let e = v.iter()
            .tuple_windows()
            .map(|(ref x, ref y)| (x.addr().unwrap(), y.addr().unwrap()))
            .collect();
        Cfg {
            verts: v.into_iter()
                .map(|x| (x.stmts[0].addr, Node::new(x)))
                .collect(),
            edges: e,
        }
    }

    fn insert_block(&mut self, block: Bb) {
        self.verts.insert(
            block.addr().unwrap(),
            Node {
                was: true,
                bb: block,
            },
        );
    }

    pub fn split(&mut self, addr: usize) -> Result<(), ()> {
        let t = *self.verts
            .iter()
            .rev()
            .find(|&(&k, _)| k <= addr)
            .ok_or(())?
            .0;
        let prev = self.verts.remove(&t).unwrap();
        match prev.bb.split(addr) {
            Ok((b1, b2)) => {
                self.insert_block(b1);
                self.insert_block(b2);
            }
            Err(b) => self.insert_block(b),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use trace::{TraceStmt, Bb};
    use cfg::Cfg;

    // TODO move it out
    macro_rules! new_trace {
        ($e:expr) => (
                TraceStmt {
                    addr: $e,
                    hex: String::new(),
                    text: String::new(),
                    isbr: false,
                    foreign: None,
                }
            )
    }

    /// Generates the CFG with the following basic blocks (one per line)
    ///  0  1  2  3
    ///  4  5  6  7
    ///  8  9 10 11
    /// 12 13 14 15
    fn make_base_cfg() -> Cfg {
        Cfg::linear(
            (0..4)
                .map(|x| {
                    Bb {
                        stmts: (0..3)
                            .map(|t| 4 * x + t)
                            .into_iter()
                            .map(|t| new_trace!(t))
                            .collect(),
                    }
                })
                .collect(),
        )
    }

    #[test]
    fn build() {
        let cfg = make_base_cfg();
        for (&v, c) in cfg.verts.keys().zip((0..4).map(|x| 4 * x)) {
            assert_eq!(v, c);
        }
        for (ref c1, ref c2) in (0..3).map(|x| (4 * x, 4 * (x + 1))) {
            assert_eq!(cfg.edges.get(c1).unwrap(), c2);
        }
    }

    #[test]
    fn split() {
        let mut cfg = make_base_cfg();
        for (&v, c) in cfg.verts.keys().zip((0..4).map(|x| 4 * x)) {
            assert_eq!(v, c);
        }
        let sz = cfg.verts.len();
        assert!(cfg.split(5).is_ok());
        assert_eq!(cfg.verts.len(), sz + 1);
        for v in vec![0, 4, 5, 8, 12] {
            assert!(cfg.verts.contains_key(&v));
        }
    }
}
