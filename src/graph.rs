extern crate dot;

use cfg::{Cfg, NodeBase};
use base::Addressable;
use std::borrow::Cow;
use std::io::Write;
use std::iter;

use itertools::Itertools;

type Node = usize;
type Edge = (usize, usize);

impl Cfg {
    pub fn render_to<W: Write>(&self, out: &mut W) {
        dot::render(self, out).unwrap()
    }
}

impl<'a> dot::Labeller<'a, Node, Edge> for Cfg {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("a").unwrap()
    }

    fn node_id(&'a self, n: &Node) -> dot::Id<'a> {
        dot::Id::new(format!("MAG{:016}MAG", self.verts[n].addr().unwrap())).unwrap()
    }

    fn node_label<'b>(&'b self, n: &Node) -> dot::LabelText<'b> {
        let ref v = self.verts[n];
        let s = match v.node {
            NodeBase::Block(ref b) => {
                let mut s = format!("{:016}\n", b.addr().unwrap());
                s.push_str(&b.instrs.iter().map(|ref x| &x.text).join("\n"));
                s
            }
            NodeBase::Foreign(ref f) => format!("{}\n", f.foreign_name),
        };
        dot::LabelText::LabelStr(Cow::Owned(s))
    }
}

impl<'a> dot::GraphWalk<'a, Node, Edge> for Cfg {
    fn nodes(&self) -> dot::Nodes<'a, usize> {
        Cow::Owned(self.verts.iter().map(|(x, _)| *x).collect())
    }

    fn edges(&'a self) -> dot::Edges<'a, Edge> {
        Cow::Owned(
            self.edges
                .iter()
                .flat_map(|(x, ref y)| {
                    iter::once(x).cartesian_product(y.iter().cloned())
                })
                .map(|(&x, y)| (x, y))
                .collect(),
        )
    }

    fn source(&self, e: &Edge) -> Node {
        let &(s, _) = e;
        s
    }

    fn target(&self, e: &Edge) -> Node {
        let &(_, t) = e;
        t
    }
}
