use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Module<'input> {
    pub imports: HashMap<&'input str, Rc<String>>,
    pub types: HashMap<&'input str, Type<'input>>,
    pub functions: HashMap<&'input str, HashMap<Vec<Rc<String>>, Function<'input>>>,
}

#[derive(Debug)]
pub struct Type<'input> {
    pub cases: Vec<TypeCase<'input>>,
}

#[derive(Debug)]
pub struct TypeCase<'input> {
    pub name: &'input str,
    pub fields: Vec<Rc<String>>,
}

#[derive(Debug)]
pub struct Function<'input> {
    pub args: Vec<ParamDef<'input>>,
    pub ret_type: Rc<String>,
    pub body: Expr<'input>,
}

#[derive(Debug)]
pub struct ParamDef<'input> {
    pub name: &'input str,
    pub param_type: Rc<String>,
}

#[derive(Debug)]
pub enum Expr<'input> {
    Const(Const),
    FuncCall(IdLoc<'input>, Vec<Rc<Expr<'input>>>),
    GetTypeCaseField(Rc<Expr<'input>>, &'input str, usize),
    If(Box<Expr<'input>>, Rc<Expr<'input>>, Box<Expr<'input>>),
    IsTypeCase(Rc<Expr<'input>>, &'input str, &'input str),
    Let(&'input str, Rc<Expr<'input>>, Rc<Expr<'input>>),
    Match(
        Box<Expr<'input>>,
        Vec<(MatchPattern<'input>, Rc<Expr<'input>>)>,
    ),
    Seq(Box<Expr<'input>>, Box<Expr<'input>>),
    TypeCase(&'input str, &'input str, Vec<Expr<'input>>),
    Var(&'input str),
}

#[derive(Debug)]
pub enum MatchPattern<'input> {
    Literal(Const),
    TypeCase(&'input str, &'input str, Vec<MatchPattern<'input>>),
    Var(&'input str),
    Wildcard,
}

#[derive(Clone, Debug)]
pub enum Const {
    Bool(bool),
    Int(u32),
    Str(Rc<String>),
}

#[derive(Debug)]
pub enum IdLoc<'input> {
    Here(&'input str),
    Other(&'input str, &'input str),
}

pub fn replace_escape_characters(str_in: String) -> Rc<String> {
    Rc::new(
        str_in
            .replace("\\\"", "\"")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\\\", "\\"),
    )
}