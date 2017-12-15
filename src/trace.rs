use itertools::Itertools;

#[derive(Debug)]
pub struct ForeignInfo {
    /// Address of the first instr after foreign branch
    pub foreign_addr: usize,
    /// Name of the foreign block
    pub foreign_name: String,
}

#[derive(Debug)]
pub struct TraceStmt {
    /// Address of the instr
    pub addr: usize,
    /// Hexdump of the instr
    pub hex: String,
    /// Disasm instr
    pub text: String,
    /// Is branch
    pub isbr: bool,
    /// Information about the foreign branch
    pub foreign: Option<ForeignInfo>,
}

#[derive(Debug)]
pub struct Bb {
    pub stmts: Vec<TraceStmt>,
}

impl Bb {
    pub fn new(stmts: Vec<TraceStmt>) -> Vec<Bb> {
        let mut tmp = stmts.into_iter().fold(vec![Vec::new()], |mut vvs, t| {
            let last = vvs.len() - 1;
            let b = t.isbr;
            vvs[last].push(t);
            if b {
                vvs.push(Vec::new());
            }
            vvs
        });
        let _ = tmp.pop();

        tmp.into_iter().map(|x| Bb { stmts: x }).collect()
    }

    pub fn split(mut self, addr: usize) -> Result<(Bb, Bb), Bb> {
        match self.stmts.iter().position(|ref x| x.addr == addr) {
            Some(i) => {
                let l = self.stmts.drain(0..i).collect();
                let r = self.stmts;
                Ok((Bb { stmts: l }, Bb { stmts: r }))
            }
            None => Err(Bb { stmts: self.stmts }),
        }
    }

    pub fn addr(&self) -> Option<usize> {
        self.stmts.iter().nth(0).map(|x| x.addr)
    }
}

#[cfg(test)]
pub mod test {
    use trace::{TraceStmt, Bb};
    use parsing::test::traces;

    #[macro_export]
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

    #[test]
    fn from_traces() {
        let bbs = Bb::new(traces());
        assert_eq!(bbs.len(), 2);
        assert_eq!(bbs[0].stmts.len(), 3);
        assert_eq!(bbs[1].stmts.len(), 4);
    }

    #[test]
    fn split() {
        let block = Bb { stmts: (10..18).map(|x| new_trace!(x)).collect() };
        let (l, r) = block.split(14).unwrap();
        let check = |bb: &Bb, rng| {
            bb.stmts.iter().map(|ref x| x.addr).zip(rng).any(
                |(x, y)| x == y,
            )
        };
        check(&l, 10..14);
        check(&r, 14..18);
    }

    #[test]
    fn addr() {
        assert_eq!(Bb { stmts: vec![new_trace!(11)] }.addr().unwrap(), 11);
    }
}
