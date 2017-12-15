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
