use std::collections::HashMap;
use std::rc::Rc;

use crate::module::Const;

#[derive(Debug)]
pub struct Binary<'input> {
    pub functions: HashMap<Rc<String>, BinFunction<'input>>,
    pub types: HashMap<Rc<String>, BinType<'input>>,
}

#[derive(Debug)]
pub struct BinType<'input> {
    pub cases: Vec<BinTypeCase<'input>>,
}

#[derive(Debug)]
pub struct BinTypeCase<'input> {
    pub name: &'input str,
    pub fields: Vec<Rc<String>>,
}

#[derive(Debug)]
pub struct BinFunction<'input> {
    pub args: Vec<BinParamDef<'input>>,
    pub ret_type: Rc<String>,
    pub body: TypedExpr<'input>,
}

#[derive(Debug)]
pub struct BinParamDef<'input> {
    pub name: &'input str,
    pub param_type: Rc<String>,
}

#[derive(Debug)]
pub struct TypedExpr<'input> {
    pub expr: BinExpr<'input>,
    pub expr_type: Rc<String>,
}

#[derive(Debug)]
pub enum BinExpr<'input> {
    Const(Const),
    FuncCall(Rc<String>, Vec<TypedExpr<'input>>),
    GetTypeCaseField(Box<TypedExpr<'input>>, &'input str, usize),
    If(
        Box<TypedExpr<'input>>,
        Box<TypedExpr<'input>>,
        Box<TypedExpr<'input>>,
    ),
    IsCase(Box<TypedExpr<'input>>, &'input str),
    Let(&'input str, Box<TypedExpr<'input>>, Box<TypedExpr<'input>>),
    Seq(Box<TypedExpr<'input>>, Box<TypedExpr<'input>>),
    TypeCase(Rc<String>, &'input str, Vec<TypedExpr<'input>>),
    Var(&'input str),
}
