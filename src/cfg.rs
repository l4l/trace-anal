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
    pub node: Node,
}

impl VisitingNode {
    fn from_node(n: Node) -> VisitingNode {
        VisitingNode { node: n }
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
    pub fn from_blocks(v: Vec<Bb>) -> Cfg {
        let nodes: Vec<(usize, VisitingNode)> = v.into_iter()
            .fold(Vec::new(), |mut acc, x| {
                let (b, f) = x.separate();
                println!("blk: {}..{}", b.addr().unwrap(), b.last().unwrap());
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
        edges.dedup();
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
        let mut cfg = Cfg {
            verts: nodes.into_iter().collect(),
            edges: edges.into_iter().fold(HashMap::new(), |mut acc, x| {
                for (l, r) in x {
                    let set = acc.entry(l).or_insert(HashSet::new());
                    set.insert(r);
                }
                acc
            }),
        };
        for addr in cfg.find_dups() {
            println!("split: {:?}", addr);
            cfg.split(addr).unwrap();
        }
        cfg
    }

    fn find_dups(&self) -> Vec<usize> {
        self.verts
            .iter()
            .filter_map(|(&x, ref y)| {
                if x == 0 {
                    return None;
                }
                if let Some(&NodeBase::Block(ref n)) =
                    self.verts.range(..x - 1).last().map(|(&_, ref y)| &y.node)
                {
                    let addr = y.addr().unwrap();
                    if let Some(s) = n.instrs.iter().find(|&x| x.addr == addr) {
                        println!("{:?}", s);
                        return Some(addr);
                    }
                }
                None
            })
            .collect()
    }

    fn insert_block(&mut self, block: Block) {
        let addr = block.addr().unwrap();
        self.verts.insert(
            addr,
            VisitingNode::from_node(NodeBase::Block(block)),
        );
    }

    pub fn split(&mut self, addr: usize) -> Result<(), ()> {
        let t = *self.verts.range(..addr).last().ok_or(())?.0;
        let prev = self.verts.remove(&t).unwrap().node;
        let prev = match prev {
            NodeBase::Block(bb) => bb,
            _ => return Err(()),
        };
        match prev.split(addr) {
            // A -> B {stmts1, stmts2} -> C
            // =>
            // A -> block1 {stmts1} -> block2 {stmts2} -> C
            Ok((block1, block2)) => {
                let (b1, b2) = (block1.addr().unwrap(), block2.addr().unwrap());
                self.insert_block(block1);
                self.insert_block(block2);

                let set = self.edges.remove(&b1).unwrap();
                // b1 -> b2
                self.edges.insert(b1, HashSet::new());
                self.edges.get_mut(&b1).ok_or(())?.insert(b2);

                // B -> C => b2 -> C
                let old = if let Some(old) = self.edges.get(&b2) {
                    old.clone()
                } else {
                    HashSet::new()
                };
                self.edges.insert(b2, old.union(&set).cloned().collect());
            }
            Err(b) => self.insert_block(b),
        }
        Ok(())
    }

    /// Performs replacing blocks `mapping` key with its value
    /// Additional redirect all the branches to the new block
    fn merge(&mut self, mapping: HashMap<usize, usize>) {
        for (&f, &t) in mapping.iter() {
            println!("merge {:?} -> {:?}", f, t);
            // Remove the corresponding vert
            self.verts.remove(&f).unwrap();
            // Replace edges: old -> _
            //            to: new -> _
            if let Some(v) = self.edges.remove(&f) {
                self.edges.insert(t, v);
            }
        }

        // Replace edges: _ -> old
        //            to: _ -> new
        for (f, t) in mapping.into_iter() {
            for (_, v) in self.edges.iter_mut() {
                if v.remove(&f) {
                    v.insert(t);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use std::collections::HashSet;
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
        Cfg::from_blocks(
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
        let vec = vec![0, 4, 5, 8, 12];
        for v in vec.iter() {
            assert!(cfg.verts.contains_key(&v));
        }
        assert_eq!(4, cfg.edges.len());
        for (c1, c2) in vec.iter().tuple_windows() {
            assert!(cfg.edges[c1].contains(c2));
        }
    }

    #[test]
    fn split_with_loop() {
        let mut cfg = make_base_cfg();
        for (&v, c) in cfg.verts.keys().zip((0..4).map(|x| 4 * x)) {
            assert_eq!(v, c);
        }
        // Before splitting
        //  0 -> 4
        //  4 -> 8,  4 -> 12
        //  8 -> 12, 8 -> 4
        // 12 -> 4
        cfg.edges.get_mut(&4).unwrap().insert(12);
        cfg.edges.get_mut(&8).unwrap().insert(4);
        cfg.edges.insert(12, HashSet::new());
        cfg.edges.get_mut(&12).unwrap().insert(4);
        // After splitting
        //  0 -> 4
        //  4 -> 5
        //  5 -> 8,  5 -> 12
        //  8 -> 12, 8 -> 4
        // 12 -> 4
        assert!(cfg.split(5).is_ok());
        let vec = vec![(0, 4), (4, 5), (5, 8), (5, 12), (8, 12), (8, 4), (12, 4)];
        assert_eq!(5, cfg.edges.len());
        for (c1, c2) in vec {
            println!("== {} {} {:?} ==", c1, c2, cfg.edges[&c1]);
            assert!(cfg.edges[&c1].contains(&c2));
        }
    }

    #[test]
    fn merge() {
        let mut cfg = make_base_cfg();
        cfg.merge((0..1).map(|_| (4, 8)).collect());
        println!("{}", cfg);
        assert_eq!(cfg.verts.len(), 3);
        assert_eq!(cfg.edges.len(), 2);
    }
}
