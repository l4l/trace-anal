use base::{Addressable, Block, Instr, ForeignInfo};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

    pub fn separate(self) -> (Block, Option<ForeignInfo>) {
        let f = self.foreign_info();
        let i = self.stmts.into_iter().map(|x| {
            Instr {
                addr: x.addr,
                hex: x.hex,
                text: x.text,
                isbr: x.isbr,
            }
        });
        (Block { instrs: i.collect() }, f)
    }

    pub fn foreign_info(&self) -> Option<ForeignInfo> {
        self.stmts.last()?.foreign.clone()
    }
}

impl Addressable for Bb {
    fn addr(&self) -> Option<usize> {
        self.stmts.iter().nth(0).map(|x| x.addr)
    }
}

#[cfg(test)]
pub mod test {
    use trace::{TraceStmt, Bb, Addressable};
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
    fn addr() {
        assert_eq!(Bb { stmts: vec![new_trace!(11)] }.addr().unwrap(), 11);
    }
}
