pub use trace::ForeignInfo;
pub use trace::TraceStmt;

extern crate simple_json;
use self::simple_json::Json;
use self::simple_json::Number::Unsigned;

use std::collections::HashMap;

macro_rules! get(
    ($e:expr) => (match $e { Some(e) => e, None => return None })
);

macro_rules! parse {
    ($e:expr, $name:expr, $opt:expr, $en:ident :: $t:ident) => (
        if let Some(& $en::$t(ref val)) = $e.get($name) {
            val
        } else {
            if let Some(ref def) = $opt {
                def
            } else {
                return None;
            }
        }
    )
}

pub fn parse_trace(s: &str) -> Vec<TraceStmt> {
    let stmts = if let Json::Array(v) = Json::parse(s).expect("Wrong json format") {
        v
    } else {
        panic!("Unsupported json, file should be wrapped in array!");
    };

    parse_stmts(stmts)
}

fn parse_addr(object: &HashMap<String, Json>, name: &str) -> Option<usize> {
    match *parse!(object, name, None, Json::Number) {
        Unsigned(v) => Some(v as usize),
        _ => None,
    }
}

fn parse_stmts(stmts: Vec<Json>) -> Vec<TraceStmt> {
    let mut parsed: Vec<TraceStmt> = Vec::new();
    for stmt in stmts {
        if let Json::Object(ref line) = stmt {
            if let Some(s) = TraceStmt::new(line) {
                parsed.push(s);
            }
        } else {
            println!("Err in parsing line: \'{}\'. Skipping...", stmt.to_string());
        }
    }
    parsed
}

impl ForeignInfo {
    fn new(object: &HashMap<String, Json>) -> Option<ForeignInfo> {
        let _ = parse!(object, "isForeignBranch", None, Json::Boolean);
        let addr = parse_addr(object, "foreignTargetAddress");
        Some(ForeignInfo {
            foreign_addr: get!(addr),
            foreign_name: String::from(
                parse!(object, "foreignTargetName", None, Json::String).as_str(),
            ),
        })
    }
}

impl TraceStmt {
    fn new(object: &HashMap<String, Json>) -> Option<TraceStmt> {
        let addr = parse_addr(object, "address");
        Some(TraceStmt {
            addr: get!(addr),
            hex: String::from(parse!(object, "hexDump", None, Json::String).as_str()),
            text: String::from(parse!(object, "text", None, Json::String).as_str()),
            isbr: *parse!(object, "isBranch", Some(false), Json::Boolean),
            foreign: ForeignInfo::new(object),
        })
    }
}

#[cfg(test)]
pub mod test {
    use parsing::*;
    use trace::TraceStmt;
    use std::cmp::Eq;
    impl PartialEq for TraceStmt {
        fn eq(&self, other: &TraceStmt) -> bool {
            self.addr == other.addr &&
            self.hex == other.hex &&
            self.text == other.text &&
            self.isbr == other.isbr &&
            self.foreign == other.foreign
        }
    }
    impl Eq for TraceStmt {}
    impl PartialEq for ForeignInfo {
        fn eq(&self, other: &ForeignInfo) -> bool {
            self.foreign_addr == other.foreign_addr &&
            self.foreign_name == other.foreign_name
        }
    }
    impl Eq for ForeignInfo {}

    fn json_traces<'a>() -> &'a str {
        r#"[{ "address": 4195392, "hexDump": "31ED", "text": "xor ebp, ebp" },
            { "address": 4195394, "hexDump": "4989D1", "text": "mov r9, rdx" },
            { "address": 4195397, "hexDump": "5E", "text": "call 0x4195398", "isBranch": true },
            { "address": 4195398, "hexDump": "4889E2", "text": "mov rdx, rsp" },
            { "address": 4195401, "hexDump": "4883E4F0", "text": "and rsp, 0xfffffffffffffff0" },
            { "address": 4195405, "hexDump": "50", "text": "push rax" },
            { "address": 4195406, "hexDump": "54", "text": "ret", "isBranch": true }]"#
    }

    pub fn traces() -> Vec<TraceStmt> {
        vec![TraceStmt{
          addr: 4195392,
          hex: "31ED".to_string(),
          text: "xor ebp, ebp".to_string(),
          isbr: false,
          foreign: None
        },
        TraceStmt{
          addr: 4195394,
          hex: "4989D1".to_string(),
          text: "mov r9, rdx".to_string(),
          isbr: false,
          foreign: None
        },
        TraceStmt{
          addr: 4195397,
          hex: "5E".to_string(),
          text: "call 0x4195398".to_string(),
          isbr: true,
          foreign: None
        },
        TraceStmt{
          addr: 4195398,
          hex: "4889E2".to_string(),
          text: "mov rdx, rsp".to_string(),
          isbr: false,
          foreign: None
        },
        TraceStmt{
          addr: 4195401,
          hex: "4883E4F0".to_string(),
          text: "and rsp, 0xfffffffffffffff0".to_string(),
          isbr: false,
          foreign: None
        },
        TraceStmt{
          addr: 4195405,
          hex: "50".to_string(),
          text: "push rax".to_string(),
          isbr: false,
          foreign: None
        },
        TraceStmt{
          addr: 4195406,
          hex: "54".to_string(),
          text: "ret".to_string(),
          isbr: true,
          foreign: None
        }]
    }

    #[test]
    fn trace_parsing() {
        let json = r#"[{ "address": 4195392, "hexDump": "31ED", "text": "xor ebp, ebp" },
                       { "address": 4195394, "hexDump": "4989D1", "text": "mov r9, rdx" },
                       { "address": 4195397, "hexDump": "5E", "text": "call 0x4195398", "isBranch": true },
                       { "address": 4195398, "hexDump": "4889E2", "text": "mov rdx, rsp" },
                       { "address": 4195401, "hexDump": "4883E4F0", "text": "and rsp, 0xfffffffffffffff0" },
                       { "address": 4195405, "hexDump": "50", "text": "push rax" },
                       { "address": 4195406, "hexDump": "54", "text": "ret", "isBranch": true }]"#;
        let trace = parse_trace(json_traces());
        for (l, r) in trace.into_iter().zip(traces()) {
            assert_eq!(l, r);
        }
    }
}
