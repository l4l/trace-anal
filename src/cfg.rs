use itertools::Itertools;
use std::collections::{HashMap, BTreeMap, HashSet};
use trace::Bb;
use base::{Addressable, Block, ForeignInfo};

#[derive(Debug)]
pub enum NodeBase<B, F> {
    Block(B),
    Foreign(F),
}

pub type Node = NodeBase<Block, ForeignInfo>;

#[derive(Debug)]
pub struct VisitingNode {
    pub was: bool,
    pub node: Node,
}

impl VisitingNode {
    fn from_node(n: Node) -> VisitingNode {
        VisitingNode {
            was: false,
            node: n,
        }
    }

    fn visit(mut self) -> VisitingNode {
        self.was = true;
        self
    }
}

impl Addressable for VisitingNode {
    fn addr(&self) -> Option<usize> {
        match self.node {
            NodeBase::Block(ref b) => b.addr(),
            NodeBase::Foreign(ref f) => f.addr(),
        }
    }
}

#[derive(Debug)]
pub struct Cfg {
    pub verts: BTreeMap<usize, VisitingNode>,
    pub edges: HashMap<usize, HashSet<usize>>,
}

impl Cfg {
    pub fn linear(v: Vec<Bb>) -> Cfg {
        let nodes: Vec<(usize, VisitingNode)> = v.into_iter()
            .fold(Vec::new(), |mut acc, x| {
                let (b, f) = x.separate();
                println!("blk: {}", b.addr().unwrap());
                acc.push((b.addr().unwrap(), NodeBase::Block(b)));
                if let Some(f) = f {
                    println!("for: {}", f.foreign_name);
                    acc.push((f.addr().unwrap(), NodeBase::Foreign(f)));
                }
                acc
            })
            .into_iter()
            .map(|(x, y)| (x, VisitingNode::from_node(y)))
            .collect();
        // Grub consequetive pairs of nodes (0,1), (1,2), ...
        let mut edges: Vec<(usize, usize)> =
            nodes.iter().map(|&(x, _)| x).tuple_windows().collect();
        edges.sort_by(|&(x, _), &(y, _)| x.cmp(&y));
        // Group edges by the source vert
        //  [0 -> 1, 0 -> 2, 1 -> 2, 3 -> 4]
        // is becoming
        //  [[0 -> 1, 0 -> 2], [1 -> 2], [3 -> 4]]
        let edges: Vec<Vec<(usize, usize)>> =
            edges.into_iter().fold(vec![Vec::new()], |mut acc, x| {
                let last = acc.len() - 1;
                if if let Some(&(t, _)) = acc[last].last() {
                    t == x.0
                } else {
                    false
                }
                {
                    acc.push(Vec::new());
                };
                acc[last].push(x);
                acc
            });
        Cfg {
            verts: nodes.into_iter().collect(),
            edges: edges.into_iter().fold(HashMap::new(), |mut acc, x| {
                for (l, r) in x {
                    let set = acc.entry(l).or_insert(HashSet::new());
                    set.insert(r);
                }
                acc
            }),
        }
    }

    fn insert_block(&mut self, block: Block) {
        let addr = block.addr().unwrap();
        self.verts.insert(
            addr,
            VisitingNode::from_node(NodeBase::Block(block)).visit(),
        );
    }

    pub fn split(&mut self, addr: usize) -> Result<(), ()> {
        let t = *self.verts
            .iter()
            .rev()
            .find(|&(&k, _)| k <= addr)
            .ok_or(())?
            .0;
        let prev = self.verts.remove(&t).unwrap().node;
        let prev = match prev {
            NodeBase::Block(bb) => bb,
            _ => return Err(()),
        };
        match prev.split(addr) {
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
            assert!(&cfg.edges[c1].contains(c2));
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
