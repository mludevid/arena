use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Module<'input> {
    pub imports: HashMap<&'input str, Rc<String>>,
    pub types: HashMap<&'input str, Type<'input>>,
    pub functions: HashMap<&'input str, HashMap<Vec<IdLoc<'input>>, Function<'input>>>,
}

#[derive(Debug)]
pub struct Type<'input> {
    pub cases: Vec<TypeCase<'input>>,
}

#[derive(Debug)]
pub struct TypeCase<'input> {
    pub name: &'input str,
    pub fields: Vec<IdLoc<'input>>,
}

#[derive(Debug)]
pub struct Function<'input> {
    pub args: Vec<ParamDef<'input>>,
    pub ret_type: IdLoc<'input>,
    pub body: Expr<'input>,
}

#[derive(Debug)]
pub struct ParamDef<'input> {
    pub name: &'input str,
    pub param_type: IdLoc<'input>,
}

#[derive(Debug)]
pub enum Expr<'input> {
    Const(Const),
    FuncCall(IdLoc<'input>, Vec<Rc<Expr<'input>>>),
    GetTypeCaseField(Rc<Expr<'input>>, &'input str, usize),
    If(Box<Expr<'input>>, Rc<Expr<'input>>, Box<Expr<'input>>),
    IsTypeCase(Rc<Expr<'input>>, IdLoc<'input>, &'input str),
    Let(&'input str, Rc<Expr<'input>>, Rc<Expr<'input>>),
    Match(
        Box<Expr<'input>>,
        Vec<(MatchPattern<'input>, Rc<Expr<'input>>)>,
    ),
    Seq(Box<Expr<'input>>, Box<Expr<'input>>),
    TypeCase(IdLoc<'input>, &'input str, Vec<Expr<'input>>),
    Var(&'input str),
}

#[derive(Debug)]
pub enum MatchPattern<'input> {
    Literal(Const),
    TypeCase(IdLoc<'input>, &'input str, Vec<MatchPattern<'input>>),
    Var(&'input str),
    Wildcard,
}

#[derive(Clone, Debug)]
pub enum Const {
    Bool(bool),
    U8(u8),
    I32(i32),
    Str(Rc<String>),
    Void,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
