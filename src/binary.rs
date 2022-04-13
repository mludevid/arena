use std::collections::HashMap;
use std::rc::Rc;

use crate::module::{Const, ParamDef, Type};

#[derive(Debug)]
pub struct Binary<'input> {
    pub functions: HashMap<Rc<String>, BinFunction<'input>>,
    pub types: HashMap<Rc<String>, Type<'input>>,
}

#[derive(Debug)]
pub struct BinFunction<'input> {
    pub args: Vec<ParamDef<'input>>,
    pub ret_type: Rc<String>,
    pub body: TypedExpr<'input>,
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
