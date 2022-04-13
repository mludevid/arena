use std::rc::Rc;

use crate::module::Module;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser, "/parser/parser.rs");

pub fn parse(input: &str) -> Module {
    parser::ModuleParser::new()
        .parse(input)
        .expect("Parsing failed")
}

pub fn parse_imports(input: &str) -> Vec<Rc<String>> {
    // TODO: Parse imports without parsing whole file
    parse(input)
        .imports
        .into_iter()
        .map(|(_, path)| path)
        .collect()
}
