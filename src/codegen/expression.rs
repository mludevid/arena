use llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

use crate::binary::BinExpr::*;
use crate::binary::*;
use crate::codegen::build_in::BuildIn::stack_alloc;
use crate::codegen::function::create_func_call;
use crate::codegen::CodegenContext;
use crate::module::Const;
use crate::module::Const::*;
use crate::types::*;

pub fn build_expression<'input>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    ast: &'input TypedExpr,
) -> *mut llvm::LLVMValue {
    match &ast.expr {
        Const(c) => build_const(cc, c),
        FuncCall(func_id, params) => {
            let mut computed_params = params
                .into_iter()
                .map(|param| build_expression(cc, current_func, vars, current_sp, &param))
                .collect::<Vec<_>>();
            create_func_call(cc, func_id, &mut computed_params, current_sp)
        }
        GetTypeCaseField(obj, case, field_index) => {
            let obj_ptr = build_expression(cc, current_func, vars, current_sp, obj);

            let (_, case_fields) = get_case_id_indices(
                cc.context,
                &cc.binary,
                &cc.llvm_structs,
                &obj.expr_type,
                case,
            );
            // Get enum field:
            let int32_type =
                type_to_llvm_type(cc.context, &cc.llvm_structs, &Rc::new(I32_TYPE.to_string()));
            let zero = unsafe { llvm::core::LLVMConstInt(int32_type, 0, 0) };
            let case_field_ptr_name = CString::new("case_field_ptr".to_string()).unwrap();
            let case_field_name = CString::new("case_field".to_string()).unwrap();
            unsafe {
                let case_field_ptr = llvm::core::LLVMBuildGEP(
                    cc.builder,
                    obj_ptr,
                    vec![zero, case_fields[*field_index]].as_mut_ptr(),
                    2,
                    case_field_ptr_name.as_ptr(),
                );
                llvm::core::LLVMBuildLoad(cc.builder, case_field_ptr, case_field_name.as_ptr())
            }
        }
        If(cond, b1, b2) => {
            let ret_type = type_to_llvm_type(cc.context, &cc.llvm_structs, &ast.expr_type);
            build_if(
                cc,
                current_func,
                vars,
                current_sp,
                &cond,
                &b1,
                &b2,
                ret_type,
            )
        }
        IsCase(obj, case) => {
            let obj_ptr = build_expression(cc, current_func, vars, current_sp, obj);

            let (case_id, _) = get_case_id_indices(
                cc.context,
                &cc.binary,
                &cc.llvm_structs,
                &obj.expr_type,
                case,
            );

            // Get enum id:
            let int32_type =
                type_to_llvm_type(cc.context, &cc.llvm_structs, &Rc::new(I32_TYPE.to_string()));
            let zero = unsafe { llvm::core::LLVMConstInt(int32_type, 0, 0) };
            let case_id_name = CString::new("case_id".to_string()).unwrap();
            unsafe {
                let id_ptr = llvm::core::LLVMBuildGEP(
                    cc.builder,
                    obj_ptr,
                    vec![zero, zero].as_mut_ptr(),
                    2,
                    case_id_name.as_ptr(),
                );
                let found_case_id =
                    llvm::core::LLVMBuildLoad(cc.builder, id_ptr, case_id_name.as_ptr());
                let cmp_name = CString::new("same_case".to_string()).unwrap();
                llvm::core::LLVMBuildICmp(
                    cc.builder,
                    llvm::LLVMIntPredicate::LLVMIntEQ,
                    case_id,
                    found_case_id,
                    cmp_name.as_ptr(),
                )
            }
        }
        Let(id, def, body) => build_let(cc, current_func, vars, current_sp, id, &def, &body),
        Seq(e1, e2) => {
            build_expression(cc, current_func, vars, current_sp, &e1);
            build_expression(cc, current_func, vars, current_sp, &e2)
        }
        TypeCase(ty, c, fields) => {
            build_type_case(cc, current_func, vars, current_sp, ty, c, fields)
        }
        Var(id) => {
            let var_name = CString::new(*id).unwrap();
            unsafe {
                llvm::core::LLVMBuildLoad(
                    cc.builder,
                    vars.get(id)
                        .unwrap_or_else(|| panic!("Could not find key {:?}", id))
                        .clone(),
                    var_name.as_ptr(),
                )
            }
        }
    }
}

fn build_const<'input>(cc: &CodegenContext, constant: &Const) -> *mut llvm::LLVMValue {
    match constant {
        Bool(b) => unsafe {
            let int1_type = type_to_llvm_type(
                cc.context,
                &cc.llvm_structs,
                &Rc::new(BOOL_TYPE.to_string()),
            );
            if *b {
                llvm::core::LLVMConstInt(int1_type, 1, 0)
            } else {
                llvm::core::LLVMConstInt(int1_type, 0, 0)
            }
        },
        U8(i) => unsafe {
            let int8_type =
                type_to_llvm_type(cc.context, &cc.llvm_structs, &Rc::new(U8_TYPE.to_string()));
            llvm::core::LLVMConstInt(int8_type, (*i as u32).into(), 0)
        },
        I32(i) => unsafe {
            let int32_type =
                type_to_llvm_type(cc.context, &cc.llvm_structs, &Rc::new(I32_TYPE.to_string()));
            llvm::core::LLVMConstInt(int32_type, (*i as u32).into(), 0)
        },
        Str(s) => unsafe {
            let c_str = CString::new(s.as_str()).unwrap();
            let name = CString::new(".str").unwrap();
            llvm::core::LLVMBuildGlobalStringPtr(cc.builder, c_str.as_ptr(), name.as_ptr())
        },
        Void => std::ptr::null_mut(),
    }
}

fn build_if<'input>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    cond: &'input TypedExpr,
    then_ast: &'input TypedExpr,
    else_ast: &'input TypedExpr,
    ret_type: *mut llvm::LLVMType,
) -> *mut llvm::LLVMValue {
    let then_name = CString::new("then").unwrap();
    let then_block = unsafe {
        llvm::core::LLVMAppendBasicBlockInContext(cc.context, current_func, then_name.as_ptr())
    };
    let else_name = CString::new("else").unwrap();
    let else_block = unsafe {
        llvm::core::LLVMAppendBasicBlockInContext(cc.context, current_func, else_name.as_ptr())
    };
    let continuation_name = CString::new("continuation").unwrap();
    let continuation_block = unsafe {
        llvm::core::LLVMAppendBasicBlockInContext(
            cc.context,
            current_func,
            continuation_name.as_ptr(),
        )
    };

    // Condition:
    let condition = build_expression(cc, current_func, vars, current_sp, cond);
    unsafe { llvm::core::LLVMBuildCondBr(cc.builder, condition, then_block, else_block) };

    // Then:
    unsafe { llvm::core::LLVMPositionBuilderAtEnd(cc.builder, then_block) };
    let then_result = build_expression(cc, current_func, vars, current_sp, then_ast);
    let incoming_block_then = unsafe { llvm::core::LLVMGetInsertBlock(cc.builder) };
    if then_ast.expr_type.as_str() != EXIT_TYPE {
        unsafe { llvm::core::LLVMBuildBr(cc.builder, continuation_block) };
    } else {
        unsafe { llvm::core::LLVMBuildBr(cc.builder, then_block) };
    }

    // Else:
    unsafe { llvm::core::LLVMPositionBuilderAtEnd(cc.builder, else_block) };
    let else_result = build_expression(cc, current_func, vars, current_sp, else_ast);
    let incoming_block_else = unsafe { llvm::core::LLVMGetInsertBlock(cc.builder) };
    if else_ast.expr_type.as_str() != EXIT_TYPE {
        unsafe { llvm::core::LLVMBuildBr(cc.builder, continuation_block) };
    } else {
        unsafe { llvm::core::LLVMBuildBr(cc.builder, else_block) };
    }

    // Continuation:
    unsafe { llvm::core::LLVMPositionBuilderAtEnd(cc.builder, continuation_block) };
    match (then_ast.expr_type.as_str(), else_ast.expr_type.as_str()) {
        (VOID_TYPE, VOID_TYPE) | (VOID_TYPE, EXIT_TYPE) | (EXIT_TYPE, VOID_TYPE) => {
            std::ptr::null_mut()
        }
        (EXIT_TYPE, _) => {
            let phi_name = CString::new("res").unwrap();
            let phi = unsafe { llvm::core::LLVMBuildPhi(cc.builder, ret_type, phi_name.as_ptr()) };
            let mut incoming_results = [else_result];
            let mut incoming_blocks = [incoming_block_else];
            unsafe {
                llvm::core::LLVMAddIncoming(
                    phi,
                    &mut incoming_results as *mut _,
                    &mut incoming_blocks as *mut _,
                    1,
                )
            };
            phi
        }
        (_, EXIT_TYPE) => {
            let phi_name = CString::new("res").unwrap();
            let phi = unsafe { llvm::core::LLVMBuildPhi(cc.builder, ret_type, phi_name.as_ptr()) };
            let mut incoming_results = [then_result];
            let mut incoming_blocks = [incoming_block_then];
            unsafe {
                llvm::core::LLVMAddIncoming(
                    phi,
                    &mut incoming_results as *mut _,
                    &mut incoming_blocks as *mut _,
                    1,
                )
            };
            phi
        }
        _ => {
            let phi_name = CString::new("res").unwrap();
            let phi = unsafe { llvm::core::LLVMBuildPhi(cc.builder, ret_type, phi_name.as_ptr()) };
            let mut incoming_results = [then_result, else_result];
            let mut incoming_blocks = [incoming_block_then, incoming_block_else];
            unsafe {
                llvm::core::LLVMAddIncoming(
                    phi,
                    &mut incoming_results as *mut _,
                    &mut incoming_blocks as *mut _,
                    2,
                )
            };
            phi
        }
    }
}

fn build_let<'input>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    id: &'input str,
    def_ast: &'input TypedExpr,
    body_ast: &'input TypedExpr,
) -> *mut llvm::LLVMValue {
    let var_name = CString::new(id).unwrap();
    let type_first_char = def_ast
        .expr_type
        .as_str()
        .chars()
        .next()
        .expect("Could not get first char of type");
    let (variable, new_sp) = if type_first_char == '$' {
        // User defined types start with $ and go on the arena stack
        let sp = create_func_call(
            cc,
            &Rc::new(stack_alloc.as_str().to_string()),
            &mut vec![current_sp],
            current_sp,
        );
        unsafe {
            (
                llvm::core::LLVMBuildBitCast(
                    cc.builder,
                    sp,
                    llvm::core::LLVMPointerType(
                        type_to_llvm_type(cc.context, &cc.llvm_structs, &def_ast.expr_type),
                        0,
                    ),
                    var_name.as_ptr(),
                ),
                sp,
            )
        }
    } else {
        // Other types go on the regular stack
        unsafe {
            (
                llvm::core::LLVMBuildAlloca(
                    cc.builder,
                    type_to_llvm_type(cc.context, &cc.llvm_structs, &def_ast.expr_type),
                    var_name.as_ptr(),
                ),
                current_sp,
            )
        }
    };
    let definition = build_expression(cc, current_func, vars, current_sp, def_ast);
    unsafe { llvm::core::LLVMBuildStore(cc.builder, definition, variable) };
    let old_def = vars.insert(id, variable);
    let res = build_expression(cc, current_func, vars, new_sp, body_ast);
    match old_def {
        None => vars.remove(&id),
        Some(def) => vars.insert(id, def),
    };
    res
}

fn build_type_case<'input>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    ty: &Rc<String>,
    case: &'input str,
    fields: &'input Vec<TypedExpr<'input>>,
) -> *mut llvm::LLVMValue {
    let llvm_type = type_to_llvm_type(cc.context, &cc.llvm_structs, ty);
    let malloc_name = CString::new(format!("{}*", ty)).unwrap();
    let heap_ptr = unsafe {
        llvm::core::LLVMBuildMalloc(
            cc.builder,
            llvm::core::LLVMGetElementType(llvm_type),
            malloc_name.as_ptr(),
        )
    };

    let (id, field_indices) =
        get_case_id_indices(cc.context, &cc.binary, &cc.llvm_structs, ty, case);

    // Save enum id:
    let int32_type =
        type_to_llvm_type(cc.context, &cc.llvm_structs, &Rc::new(I32_TYPE.to_string()));
    let zero = unsafe { llvm::core::LLVMConstInt(int32_type, 0, 0) };
    let save_id_name = CString::new("case_id".to_string()).unwrap();
    unsafe {
        let ptr = llvm::core::LLVMBuildGEP(
            cc.builder,
            heap_ptr,
            vec![zero, zero].as_mut_ptr(),
            2,
            save_id_name.as_ptr(),
        );
        llvm::core::LLVMBuildStore(cc.builder, id, ptr)
    };

    // Save fields:
    for (expr, index) in fields.iter().zip(field_indices.into_iter()) {
        let field = build_expression(cc, current_func, vars, current_sp, expr);
        let field_ptr = CString::new("field_ptr".to_string()).unwrap();
        unsafe {
            let ptr = llvm::core::LLVMBuildGEP(
                cc.builder,
                heap_ptr,
                vec![zero, index].as_mut_ptr(),
                2,
                field_ptr.as_ptr(),
            );
            llvm::core::LLVMBuildStore(cc.builder, field, ptr);
        }
    }

    heap_ptr
}
