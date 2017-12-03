extern crate itertools;
use self::itertools::Itertools;

#[derive(Debug)]
pub struct ForeignInfo {
    /// Address of the first instr after foreign branch
    pub foreign_addr: u64,
    /// Name of the foreign block
    pub foreign_name: String,
}

#[derive(Debug)]
pub struct TraceStmt {
    /// Address of the instr
    pub addr: u64,
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
        stmts
            .into_iter()
            .group_by(|x| x.isbr)
            .into_iter()
            .map(|(_, x)| Bb { stmts: x.collect() })
            .collect()
    }
}
