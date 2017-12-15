pub trait Addressable {
    fn addr(&self) -> Option<usize>;
}

#[derive(Debug, Clone)]
pub struct ForeignInfo {
    /// Address of the first instr after foreign branch
    pub foreign_addr: usize,
    /// Name of the foreign block
    pub foreign_name: String,
}

#[derive(Debug)]
pub struct Instr {
    /// Address of the instr
    pub addr: usize,
    /// Hexdump of the instr
    pub hex: String,
    /// Disasm instr
    pub text: String,
    /// Is branch
    pub isbr: bool,
}

#[derive(Debug)]
pub struct Block {
    pub instrs: Vec<Instr>,
}

impl Addressable for ForeignInfo {
    fn addr(&self) -> Option<usize> {
        Some(self.foreign_addr)
    }
}

impl Addressable for Block {
    fn addr(&self) -> Option<usize> {
        self.instrs.iter().nth(0).map(|x| x.addr)
    }
}

impl Block {
    pub fn split(mut self, addr: usize) -> Result<(Block, Block), Block> {
        match self.instrs.iter().position(|ref x| x.addr == addr) {
            Some(i) => {
                let l = self.instrs.drain(0..i).collect();
                let r = self.instrs;
                Ok((Block { instrs: l }, Block { instrs: r }))
            }
            None => Err(Block { instrs: self.instrs }),
        }
    }
}

#[cfg(test)]
mod test {
    use base::{Block, Instr};
    // TODO move it out
    macro_rules! new_instr {
        ($e:expr) => (
                Instr {
                    addr: $e,
                    hex: String::new(),
                    text: String::new(),
                    isbr: false,
                }
            )
    }

    #[test]
    fn split() {
        let block = Block { instrs: (10..18).map(|x| new_instr!(x)).collect() };
        let (l, r) = block.split(14).unwrap();
        let check = |bb: &Block, rng| {
            bb.instrs.iter().map(|ref x| x.addr).zip(rng).any(
                |(x, y)| x == y,
            )
        };
        check(&l, 10..14);
        check(&r, 14..18);
    }
}
