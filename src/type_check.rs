use std::collections::HashMap;
use std::rc::Rc;

use crate::binary::*;
use crate::codegen::build_in::get_build_in_signature;
use crate::module::*;
use crate::types::*;

pub fn type_check<'input>(
    modules: &'input HashMap<Rc<String>, Module<'input>>,
    main_module_id: &Rc<String>,
) -> Result<Binary<'input>, String> {
    // GENERATE UNIQUE TYPE NAMES:
    // type_names[module_id][type_name]
    let mut type_names: HashMap<&Rc<String>, HashMap<&str, Rc<String>>> = HashMap::new();
    for (unique_name, module) in modules.iter() {
        let mut module_types = HashMap::new();
        for (name, _) in module.types.iter() {
            let type_name = Rc::new(format!("{}{}", unique_name, name));
            module_types.insert(*name, Rc::clone(&type_name));
        }
        type_names.insert(unique_name, module_types);
    }
    // UNIFY ALL TYPES AND FIX FIELD TYPES
    let mut all_types: HashMap<Rc<String>, Type> = HashMap::new();
    for (unique_name, module) in modules.iter() {
        for (name, t) in module.types.iter() {
            // TODO: Check that no type has VOID as a field
            all_types.insert(
                Rc::clone(&type_names[unique_name][name]),
                Type {
                    cases: t
                        .cases
                        .iter()
                        .map(|case| TypeCase {
                            name: case.name,
                            fields: case
                                .fields
                                .iter()
                                .map(|f| get_unique_type_id(&type_names, unique_name, f))
                                .collect(),
                        })
                        .collect(),
                },
            );
        }
    }

    // GENERATE UNIQUE FUNCTION NAMES:
    // function_names[module_id][function_name][parameter_types]
    // function_defs[function_id]
    let mut function_names: HashMap<
        &Rc<String>,
        HashMap<&str, HashMap<Vec<Rc<String>>, Rc<String>>>,
    > = HashMap::new();
    let mut function_defs: HashMap<Rc<String>, &Function> = HashMap::new();
    for (unique_name, module) in modules.iter() {
        let mut module_function = HashMap::new();
        for (name, functions) in module.functions.iter() {
            let mut polymorph_functions = HashMap::new();
            for (index, (signature, f)) in functions.iter().enumerate() {
                let function_id = Rc::new(format!("{}{}${}", unique_name, name, index));
                polymorph_functions.insert(
                    signature
                        .iter()
                        .map(|ty| get_unique_type_id(&type_names, unique_name, ty))
                        .collect(),
                    Rc::clone(&function_id),
                );
                function_defs.insert(function_id, f);
            }
            module_function.insert(*name, polymorph_functions);
        }
        function_names.insert(unique_name, module_function);
    }

    let mut checked_functions = HashMap::new();

    for (unique_name, module) in modules.iter() {
        for (name, functions) in module.functions.iter() {
            for (signature, function) in functions.iter() {
                let sig: Vec<Rc<String>> = signature
                    .iter()
                    .map(|ty| get_unique_type_id(&type_names, unique_name, ty))
                    .collect();
                checked_functions.insert(
                    Rc::clone(&function_names[unique_name][name][&sig]),
                    type_check_function(
                        unique_name,
                        &module.imports,
                        &function_names,
                        &function_defs,
                        &type_names,
                        &all_types,
                        function,
                    )?, // TODO: Improve Error handling by concatenating them
                );
            }
        }
    }

    let main_module = function_names
        .get(main_module_id)
        .ok_or("Could not find main module")?;
    let main_function_name = main_module
        .get("main")
        .ok_or("Could not find function named main")?;
    let main_function = main_function_name
        .get(&vec![])
        .ok_or("Could not find main function with no arguments")?;
    let inserted_main_functions = type_check_main(checked_functions, main_function)?;

    Ok(Binary {
        functions: inserted_main_functions,
        types: all_types,
    })
}

fn type_check_main<'input>(
    mut functions: HashMap<Rc<String>, BinFunction<'input>>,
    main_function: &Rc<String>,
) -> Result<HashMap<Rc<String>, BinFunction<'input>>, String> {
    // Check if main function returns void:
    let main_func = functions
        .get(main_function)
        .ok_or("Could not find main function".to_string())?;
    if main_func.ret_type.as_str() != VOID_TYPE {
        return Err("Main function has to return void".to_string());
    }

    let int_type = Rc::new(I32_TYPE.to_string());
    // Insert LLVM main function that calls user defined main function
    let main_calling_func = BinFunction {
        args: Vec::new(),
        ret_type: Rc::clone(&int_type),
        body: TypedExpr {
            expr: BinExpr::Seq(
                Box::new(TypedExpr {
                    expr: BinExpr::FuncCall(Rc::clone(main_function), Vec::new()),
                    expr_type: Rc::new(VOID_TYPE.to_string()),
                }),
                Box::new(TypedExpr {
                    expr: BinExpr::Const(Const::Int(0)),
                    expr_type: Rc::clone(&int_type),
                }),
            ),
            expr_type: int_type,
        },
    };
    functions.insert(Rc::new("main".to_string()), main_calling_func);
    Ok(functions)
}

fn type_check_function<'input>(
    module_id: &Rc<String>,
    imports: &HashMap<&str, Rc<String>>,
    function_ids: &HashMap<&Rc<String>, HashMap<&'input str, HashMap<Vec<Rc<String>>, Rc<String>>>>,
    function_defs: &HashMap<Rc<String>, &Function>,
    type_ids: &HashMap<&Rc<String>, HashMap<&'input str, Rc<String>>>,
    type_defs: &HashMap<Rc<String>, Type>,
    function: &Function<'input>,
) -> Result<BinFunction<'input>, String> {
    let args = function
        .args
        .iter()
        .map(|arg| ParamDef {
            name: arg.name,
            param_type: get_unique_type_id(type_ids, module_id, &arg.param_type),
        })
        .collect::<Vec<_>>();
    let mut vars: HashMap<&'input str, Rc<String>> = HashMap::new();
    for arg in args.iter() {
        if arg.param_type.as_str() == VOID_TYPE {
            return Err("Function parameter type Void is prohibited".to_string());
        }
        vars.insert(arg.name, Rc::clone(&arg.param_type));
    }

    let typed_body = type_check_expr(
        module_id,
        imports,
        function_ids,
        function_defs,
        type_ids,
        type_defs,
        &mut vars,
        &function.body,
    )?;
    let ret_type = get_unique_type_id(type_ids, module_id, &function.ret_type);
    let checked_body = if ret_type.as_str() == VOID_TYPE {
        // Functions that return VOID will drop any value therefore it does not
        // have to be a void, it can be any value
        typed_body
    } else {
        expect_type(ret_type.as_str(), typed_body)?
    };
    Ok(BinFunction {
        args,
        ret_type,
        body: checked_body,
    })
}

fn type_check_expr<'input>(
    module_id: &Rc<String>,
    imports: &HashMap<&str, Rc<String>>,
    function_ids: &HashMap<&Rc<String>, HashMap<&'input str, HashMap<Vec<Rc<String>>, Rc<String>>>>,
    function_defs: &HashMap<Rc<String>, &Function>,
    type_ids: &HashMap<&Rc<String>, HashMap<&'input str, Rc<String>>>,
    type_defs: &HashMap<Rc<String>, Type>,
    vars: &mut HashMap<&'input str, Rc<String>>,
    expr: &Expr<'input>,
) -> Result<TypedExpr<'input>, String> {
    Ok(match expr {
        Expr::Const(Const::Int(i)) => TypedExpr {
            expr: BinExpr::Const(Const::Int(*i)),
            expr_type: Rc::new(I32_TYPE.to_string()),
        },
        Expr::Const(Const::Bool(b)) => TypedExpr {
            expr: BinExpr::Const(Const::Bool(*b)),
            expr_type: Rc::new(BOOL_TYPE.to_string()),
        },
        Expr::Const(Const::Str(s)) => TypedExpr {
            expr: BinExpr::Const(Const::Str(Rc::clone(s))),
            expr_type: Rc::new(STRING_TYPE.to_string()),
        },
        Expr::FuncCall(id_loc, args) => type_check_func_call(
            module_id,
            imports,
            function_ids,
            function_defs,
            type_ids,
            type_defs,
            vars,
            id_loc,
            args,
        )?,
        Expr::GetTypeCaseField(obj, case, field_index) => {
            let typed_obj = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                obj,
            )?;
            let case_def = type_defs[&typed_obj.expr_type]
                .cases
                .iter()
                .find(|f| f.name == *case)
                .ok_or(format!(
                    "Could not find case {} for {}",
                    case,
                    typed_obj.expr_type.as_str()
                ))?;
            let typed_obj_type = Rc::clone(&typed_obj.expr_type);
            TypedExpr {
                expr: BinExpr::GetTypeCaseField(Box::new(typed_obj), case, *field_index),
                expr_type: Rc::clone(case_def.fields.get(*field_index).ok_or(format!(
                    "Index {} out of bound for {}.{}",
                    field_index,
                    typed_obj_type.as_str(),
                    case_def.name
                ))?),
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            let typed_cond = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                cond,
            )?;
            let checked_cond = expect_type(BOOL_TYPE, typed_cond)?;
            let typed_then = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                then_expr,
            )?;
            let typed_else = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                else_expr,
            )?;
            let checked_else = expect_type(typed_then.expr_type.as_str(), typed_else)?;
            let ret_type = if typed_then.expr_type.as_str() == EXIT_TYPE {
                Rc::clone(&checked_else.expr_type)
            } else {
                Rc::clone(&typed_then.expr_type)
            };
            TypedExpr {
                expr: BinExpr::If(
                    Box::new(checked_cond),
                    Box::new(typed_then),
                    Box::new(checked_else),
                ),
                expr_type: ret_type,
            }
        }
        Expr::IsTypeCase(ty_case, ty, case) => {
            let typed_ty_case = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                ty_case,
            )?;
            let type_id = match ty {
                IdLoc::Here(typ_name) => type_ids[module_id]
                    .get(typ_name)
                    .ok_or(format!("Type {} not found", typ_name))?,
                IdLoc::Other(mod_name, typ_name) => match imports.get(mod_name) {
                    Some(other_mod) => type_ids[other_mod]
                        .get(typ_name)
                        .ok_or(format!("Type {} not found", typ_name))?,
                    None => Err(format!("Unresolved import: {}", mod_name))?,
                },
            };
            if typed_ty_case.expr_type.as_str() == type_id.as_str() {
                if type_defs[type_id]
                    .cases
                    .iter()
                    .find(|c| c.name == *case)
                    .is_some()
                {
                    TypedExpr {
                        expr: BinExpr::IsCase(Box::new(typed_ty_case), case),
                        expr_type: Rc::new(BOOL_TYPE.to_string()),
                    }
                } else {
                    return Err(format!("{} is not a case of {}", case, type_id.as_str(),));
                }
            } else {
                return Err(format!(
                    "Can not match type {} to type {}",
                    type_id.as_str(),
                    typed_ty_case.expr_type.as_str()
                ));
            }
        }
        Expr::Let(name, definition, body) => type_check_let(
            module_id,
            imports,
            function_ids,
            function_defs,
            type_ids,
            type_defs,
            vars,
            name,
            definition.as_ref(),
            body.as_ref(),
        )?,
        Expr::Match(obj, match_arms) => type_check_match(
            module_id,
            imports,
            function_ids,
            function_defs,
            type_ids,
            type_defs,
            vars,
            obj,
            match_arms,
        )?,
        Expr::Seq(e1, e2) => {
            let checked_e1 = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                e1.as_ref(),
            )?;
            let checked_e2 = type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                e2.as_ref(),
            )?;
            let ret_type = Rc::clone(&checked_e2.expr_type);
            TypedExpr {
                expr: BinExpr::Seq(Box::new(checked_e1), Box::new(checked_e2)),
                expr_type: ret_type,
            }
        }
        Expr::TypeCase(typ, case, args) => {
            let type_id = match typ {
                IdLoc::Here(typ_name) => type_ids[module_id]
                    .get(typ_name)
                    .ok_or(format!("Type {} not found", typ_name))?,
                IdLoc::Other(mod_name, typ_name) => match imports.get(mod_name) {
                    Some(other_mod) => type_ids[other_mod]
                        .get(typ_name)
                        .ok_or(format!("Type {} not found", typ_name))?,
                    None => Err(format!("Unresolved import: {}", mod_name))?,
                },
            };
            let type_def = &type_defs[type_id];
            let case_def = type_def
                .cases
                .iter()
                .find(|def| def.name == *case)
                .expect("Could not find type case definition");
            if case_def.fields.len() != args.len() {
                return Err(format!(
                    "{}.{} expects {} fields, got {}",
                    type_id.as_str(),
                    case,
                    case_def.fields.len(),
                    args.len()
                ));
            }
            let checked_args = args
                .iter()
                .zip(case_def.fields.iter())
                .map(|(arg, arg_def)| {
                    expect_type(
                        arg_def.as_str(),
                        type_check_expr(
                            module_id,
                            imports,
                            function_ids,
                            function_defs,
                            type_ids,
                            type_defs,
                            vars,
                            arg,
                        )?,
                    )
                })
                .collect::<Result<Vec<_>, String>>()?; // TODO: Improve error handling by joining all Error Strings
            let ret_type = Rc::clone(&type_id);
            TypedExpr {
                expr: BinExpr::TypeCase(Rc::clone(&type_id), case, checked_args),
                expr_type: ret_type,
            }
        }
        Expr::Var(name) => match vars.get(name) {
            Some(var_type) => TypedExpr {
                expr: BinExpr::Var(name),
                expr_type: Rc::clone(&var_type),
            },
            None => {
                return Err(format!("Variable {} not found", name));
            }
        },
    })
}

fn type_check_func_call<'input>(
    module_id: &Rc<String>,
    imports: &HashMap<&str, Rc<String>>,
    function_ids: &HashMap<&Rc<String>, HashMap<&'input str, HashMap<Vec<Rc<String>>, Rc<String>>>>,
    function_defs: &HashMap<Rc<String>, &Function>,
    type_ids: &HashMap<&Rc<String>, HashMap<&'input str, Rc<String>>>,
    type_defs: &HashMap<Rc<String>, Type>,
    vars: &mut HashMap<&'input str, Rc<String>>,
    id_loc: &IdLoc<'input>,
    args: &Vec<Rc<Expr<'input>>>,
) -> Result<TypedExpr<'input>, String> {
    let type_checked_args = args
        .iter()
        .map(|arg| {
            type_check_expr(
                module_id,
                imports,
                function_ids,
                function_defs,
                type_ids,
                type_defs,
                vars,
                arg.as_ref(),
            )
        })
        .collect::<Result<Vec<_>, String>>()?; // TODO: Improve error handling by joining all Error Strings

    let arg_types = type_checked_args
        .iter()
        .map(|arg| Rc::clone(&arg.expr_type))
        .collect::<Vec<_>>();

    // Find function:
    match id_loc {
        IdLoc::Here(name) => {
            // Find function in own module
            match function_ids[module_id].get(name) {
                Some(functions) => {
                    // There are some functions with this name. Check for matching signature
                    match functions.get(&arg_types) {
                        Some(function_id) => {
                            let function = function_defs[function_id];
                            let ret_type =
                                get_unique_type_id(type_ids, module_id, &function.ret_type);
                            return Ok(TypedExpr {
                                expr: BinExpr::FuncCall(Rc::clone(&function_id), type_checked_args),
                                expr_type: ret_type,
                            });
                        }
                        None => (),
                    }
                }
                None => (),
            }

            // If not found search build_in function
            match get_build_in_signature(&name, &arg_types) {
                Some((func_call_name, ret_type)) => Ok(TypedExpr {
                    expr: BinExpr::FuncCall(func_call_name, type_checked_args),
                    expr_type: ret_type,
                }),
                None => Err(format!(
                    "Could not find function {} with signature {:?}",
                    &name, &arg_types
                )),
            }
        }
        IdLoc::Other(module, name) => {
            match imports.get(module) {
                Some(other_module_id) => match function_ids[other_module_id].get(name) {
                    Some(functions) => {
                        // There are some functions with this name. Check for matching signature
                        match functions.get(&arg_types) {
                            Some(function_id) => {
                                let function = function_defs[function_id];
                                let ret_type = Rc::clone(&function.ret_type);
                                Ok(TypedExpr {
                                    expr: BinExpr::FuncCall(
                                        Rc::clone(function_id),
                                        type_checked_args,
                                    ),
                                    expr_type: ret_type,
                                })
                            }
                            None => Err(format!(
                                "Could not find function {}::{} with signature {:?}",
                                module, name, arg_types
                            )),
                        }
                    }
                    None => Err(format!(
                        "Could not find function {}::{} with signature {:?}",
                        module, name, arg_types
                    )),
                },
                None => Err(format!("Unresolved import: {}", module)),
            }
        }
    }
}

fn type_check_let<'input>(
    module_id: &Rc<String>,
    imports: &HashMap<&str, Rc<String>>,
    function_ids: &HashMap<&Rc<String>, HashMap<&'input str, HashMap<Vec<Rc<String>>, Rc<String>>>>,
    function_defs: &HashMap<Rc<String>, &Function>,
    type_ids: &HashMap<&Rc<String>, HashMap<&'input str, Rc<String>>>,
    type_defs: &HashMap<Rc<String>, Type>,
    vars: &mut HashMap<&'input str, Rc<String>>,
    name: &'input str,
    definition: &Expr<'input>,
    body: &Expr<'input>,
) -> Result<TypedExpr<'input>, String> {
    let typed_def = type_check_expr(
        module_id,
        imports,
        function_ids,
        function_defs,
        type_ids,
        type_defs,
        vars,
        definition,
    )?;
    if typed_def.expr_type.as_str() == VOID_TYPE {
        return Err("Variables of type void are not allowed".to_string());
    }
    let old_type = vars.insert(name, Rc::clone(&typed_def.expr_type));
    let typed_body = type_check_expr(
        module_id,
        imports,
        function_ids,
        function_defs,
        type_ids,
        type_defs,
        vars,
        body,
    )?;
    let ret_type = Rc::clone(&typed_body.expr_type);
    match old_type {
        None => vars.remove(&name),
        Some(t) => vars.insert(name, t),
    };
    Ok(TypedExpr {
        expr: BinExpr::Let(name, Box::new(typed_def), Box::new(typed_body)),
        expr_type: ret_type,
    })
}

fn type_check_match<'input>(
    module_id: &Rc<String>,
    imports: &HashMap<&str, Rc<String>>,
    function_ids: &HashMap<&Rc<String>, HashMap<&'input str, HashMap<Vec<Rc<String>>, Rc<String>>>>,
    function_defs: &HashMap<Rc<String>, &Function>,
    type_ids: &HashMap<&Rc<String>, HashMap<&'input str, Rc<String>>>,
    type_defs: &HashMap<Rc<String>, Type>,
    vars: &mut HashMap<&'input str, Rc<String>>,
    obj: &Box<Expr<'input>>,
    match_arms: &Vec<(MatchPattern<'input>, Rc<Expr<'input>>)>,
) -> Result<TypedExpr<'input>, String> {
    type_check_let(
        module_id,
        imports,
        function_ids,
        function_defs,
        type_ids,
        type_defs,
        vars,
        "$match_obj$",
        obj.as_ref(),
        &build_ifs_from_arms(match_arms),
    )
}

fn build_ifs_from_arms<'input>(
    match_arms: &Vec<(MatchPattern<'input>, Rc<Expr<'input>>)>,
) -> Expr<'input> {
    let failed_expr = Expr::Seq(
        Box::new(Expr::FuncCall(
            IdLoc::Here("print"),
            vec![Rc::new(Expr::Const(Const::Str(Rc::new(
                "Not exhaustive match\n".to_string(),
            ))))],
        )),
        Box::new(Expr::FuncCall(
            IdLoc::Here("exit"),
            vec![Rc::new(Expr::Const(Const::Int(1)))],
        )),
    );
    let ifs = match_arms.iter().fold(failed_expr, |acc, arm| {
        let (condition, variable_definitions) =
            get_condition_vars_pattern(Rc::new(Expr::Var("$match_obj$")), &arm.0);
        let then_expr = variable_definitions
            .into_iter()
            .rev()
            .fold(Rc::clone(&arm.1), |body, var_def| {
                Rc::new(Expr::Let(var_def.0, var_def.1, Rc::clone(&body)))
            });
        Expr::If(Box::new(condition), then_expr, Box::new(acc))
    });
    ifs
}

fn get_condition_vars_pattern<'input>(
    obj: Rc<Expr<'input>>,
    pattern: &MatchPattern<'input>,
) -> (Expr<'input>, Vec<(&'input str, Rc<Expr<'input>>)>) {
    match pattern {
        MatchPattern::Literal(c) => (
            Expr::FuncCall(
                IdLoc::Here("eq"),
                vec![Rc::new(Expr::Const(c.clone())), obj],
            ),
            Vec::new(),
        ),
        MatchPattern::TypeCase(ty, case, fields) => fields.into_iter().enumerate().fold(
            (
                Expr::IsTypeCase(Rc::clone(&obj), ty.clone(), case),
                Vec::new(),
            ),
            |(acc_expr, mut acc_vars), (i, field)| {
                let (cond, mut vars) = get_condition_vars_pattern(
                    Rc::new(Expr::GetTypeCaseField(Rc::clone(&obj), case, i)),
                    field,
                );
                acc_vars.append(&mut vars);
                (
                    Expr::If(
                        Box::new(acc_expr),
                        Rc::new(cond),
                        Box::new(Expr::Const(Const::Bool(false))),
                    ),
                    acc_vars,
                )
            },
        ),
        MatchPattern::Var(name) => (Expr::Const(Const::Bool(true)), vec![(*name, obj)]),
        MatchPattern::Wildcard => (Expr::Const(Const::Bool(true)), Vec::new()),
    }
}

fn get_unique_type_id(
    type_ids: &HashMap<&Rc<String>, HashMap<&str, Rc<String>>>,
    module_id: &Rc<String>,
    ret_type: &Rc<String>,
) -> Rc<String> {
    Rc::clone(
        type_ids[module_id]
            .get(ret_type.as_str())
            .unwrap_or(ret_type),
    )
}

fn expect_type<'input>(
    type_expected: &str,
    expr: TypedExpr<'input>,
) -> Result<TypedExpr<'input>, String> {
    if type_expected == EXIT_TYPE || expr.expr_type.as_str() == EXIT_TYPE {
        return Ok(expr);
    }
    if type_expected == expr.expr_type.as_str() {
        Ok(expr)
    } else {
        Err(format!(
            "Expected {}, but found {}",
            type_expected, expr.expr_type
        ))
    }
}
