use llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

use crate::binary::BinExpr::*;
use crate::binary::*;
use crate::codegen::build_in::BuildIn::stack_alloc;
use crate::codegen::function::create_func_call;
use crate::codegen::garbage_collection::GC;
use crate::codegen::CodegenContext;
use crate::module::Const;
use crate::module::Const::*;
use crate::types::*;

pub fn build_expression<'input, Gc: GC>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    ast: &'input TypedExpr,
) -> *mut llvm::LLVMValue {
    match &ast.expr {
        Const(c) => build_const(cc, c),
        FuncCall(func_id, params) => {
            let (computed_params, stored_params, new_sp) = compute_params::<Gc>(cc, current_func, vars, current_sp, params);
            let mut loaded_params = load_params(cc, computed_params, stored_params);
            create_func_call::<Gc>(cc, func_id, &mut loaded_params, new_sp)
        }
        GetTypeCaseField(obj, case, field_index) => {
            let obj_ptr = build_expression::<Gc>(cc, current_func, vars, current_sp, obj);

            let (_, case_fields, _) = get_case_id_case_indices_pointer_indices::<Gc>(
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
            let case_field = unsafe {
                let case_field_ptr = llvm::core::LLVMBuildGEP(
                    cc.builder,
                    obj_ptr,
                    vec![zero, case_fields[*field_index]].as_mut_ptr(),
                    2,
                    case_field_ptr_name.as_ptr(),
                );
                llvm::core::LLVMBuildLoad(cc.builder, case_field_ptr, case_field_name.as_ptr())
            };
            let type_first_char = ast
                .expr_type
                .as_str()
                .chars()
                .next()
                .expect("Could not get first char of type");
            if type_first_char == '$' {
                // User defined type:
                Gc::type_ptr_access(cc, case_field);
            }
            Gc::type_ptr_drop(cc, obj_ptr);
            case_field
        }
        If(cond, b1, b2) => {
            let ret_type = type_to_llvm_type(cc.context, &cc.llvm_structs, &ast.expr_type);
            build_if::<Gc>(
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
            let obj_ptr = build_expression::<Gc>(cc, current_func, vars, current_sp, obj);

            let (case_id, _, _) = get_case_id_case_indices_pointer_indices::<Gc>(
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
            let case_id_index =
                unsafe { llvm::core::LLVMConstInt(int32_type, Gc::get_type_header_length(), 0) };
            let case_id_name = CString::new("case_id".to_string()).unwrap();
            unsafe {
                let id_ptr = llvm::core::LLVMBuildGEP(
                    cc.builder,
                    obj_ptr,
                    vec![zero, case_id_index].as_mut_ptr(),
                    2,
                    case_id_name.as_ptr(),
                );
                let found_case_id =
                    llvm::core::LLVMBuildLoad(cc.builder, id_ptr, case_id_name.as_ptr());
                let cmp_name = CString::new("same_case".to_string()).unwrap();
                let cmp = llvm::core::LLVMBuildICmp(
                    cc.builder,
                    llvm::LLVMIntPredicate::LLVMIntEQ,
                    case_id,
                    found_case_id,
                    cmp_name.as_ptr(),
                );
                Gc::type_ptr_drop(cc, obj_ptr);
                cmp
            }
        }
        Let(id, def, body) => build_let::<Gc>(cc, current_func, vars, current_sp, id, &def, &body),
        Seq(e1, e2) => {
            let e1_res = build_expression::<Gc>(cc, current_func, vars, current_sp, &e1);
            let type_first_char = e1
                .expr_type
                .as_str()
                .chars()
                .next()
                .expect("Could not get first char of type");
            if type_first_char == '$' {
                // User defined type:
                Gc::type_ptr_drop(cc, e1_res);
            }
            build_expression::<Gc>(cc, current_func, vars, current_sp, &e2)
        }
        TypeCase(ty, c, fields) => {
            build_type_case::<Gc>(cc, current_func, vars, current_sp, ty, c, fields)
        }
        Var(id) => {
            let var_name = CString::new(*id).unwrap();
            let var = unsafe {
                llvm::core::LLVMBuildLoad(
                    cc.builder,
                    vars.get(id)
                        .unwrap_or_else(|| panic!("Could not find key {:?}", id))
                        .clone(),
                    var_name.as_ptr(),
                )
            };
            let type_first_char = ast
                .expr_type
                .as_str()
                .chars()
                .next()
                .expect("Could not get first char of type");
            if type_first_char == '$' {
                // User defined type:
                Gc::type_ptr_access(cc, var);
            }
            var
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

fn build_if<'input, Gc: GC>(
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
    let condition = build_expression::<Gc>(cc, current_func, vars, current_sp, cond);
    unsafe { llvm::core::LLVMBuildCondBr(cc.builder, condition, then_block, else_block) };

    // Then:
    unsafe { llvm::core::LLVMPositionBuilderAtEnd(cc.builder, then_block) };
    let then_result = build_expression::<Gc>(cc, current_func, vars, current_sp, then_ast);
    let incoming_block_then = unsafe { llvm::core::LLVMGetInsertBlock(cc.builder) };
    if then_ast.expr_type.as_str() != EXIT_TYPE {
        unsafe { llvm::core::LLVMBuildBr(cc.builder, continuation_block) };
    } else {
        unsafe { llvm::core::LLVMBuildBr(cc.builder, then_block) };
    }

    // Else:
    unsafe { llvm::core::LLVMPositionBuilderAtEnd(cc.builder, else_block) };
    let else_result = build_expression::<Gc>(cc, current_func, vars, current_sp, else_ast);
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

fn get_next_stack_element<Gc: GC>(
    current_sp: *mut llvm::LLVMValue,
    cc: &CodegenContext,
    var_name: &CString,
    var_type: &Rc<String>,
) -> (*mut llvm::LLVMValue, *mut llvm::LLVMValue) {
    let sp = create_func_call::<Gc>(
        cc,
        &Rc::new(stack_alloc.as_str().to_string()),
        &mut vec![current_sp, cc.profiling_frequency],
        current_sp,
    );
    unsafe {
        (
            llvm::core::LLVMBuildBitCast(
                cc.builder,
                sp,
                llvm::core::LLVMPointerType(
                    type_to_llvm_type(cc.context, &cc.llvm_structs, var_type),
                    0,
                ),
                var_name.as_ptr(),
            ),
            sp,
        )
    }
}

fn build_let<'input, Gc: GC>(
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
        get_next_stack_element::<Gc>(current_sp, cc, &var_name, &def_ast.expr_type)
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
    let definition = build_expression::<Gc>(cc, current_func, vars, current_sp, def_ast);
    unsafe { llvm::core::LLVMBuildStore(cc.builder, definition, variable) };
    let old_def = vars.insert(id, variable);
    let res = build_expression::<Gc>(cc, current_func, vars, new_sp, body_ast);
    if type_first_char == '$' {
        // User defined type:
        // Gc::type_ptr_drop(cc, definition);
        // let stack_ptr =
        // unsafe { llvm::core::LLVMBuildLoad(cc.builder, variable, var_name.as_ptr()) };
        let obj_ptr = unsafe { llvm::core::LLVMBuildLoad(cc.builder, variable, var_name.as_ptr()) };
        Gc::type_ptr_drop(cc, obj_ptr);
    }
    match old_def {
        None => vars.remove(&id),
        Some(def) => vars.insert(id, def),
    };
    res
}

fn build_type_case<'input, Gc: GC>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    ty: &Rc<String>,
    case: &'input str,
    fields: &'input Vec<TypedExpr<'input>>,
) -> *mut llvm::LLVMValue {
    let (computed_params, stored_params, sp) = compute_params::<Gc>(cc, current_func, vars, current_sp, fields);

    let llvm_type = type_to_llvm_type(cc.context, &cc.llvm_structs, ty);
    let size = get_struct_size(&cc.llvm_structs, ty);
    let malloc_ret = Gc::type_allocation(cc, size, sp);
    let struct_name = CString::new(format!("{}*", ty)).unwrap();
    let heap_ptr = unsafe {
        llvm::core::LLVMBuildBitCast(cc.builder, malloc_ret, llvm_type, struct_name.as_ptr())
    };

    let fs = load_params(cc, computed_params, stored_params);

    Gc::init_header(cc, heap_ptr, size);

    let (id, field_indices, pointer_indices) = get_case_id_case_indices_pointer_indices::<Gc>(
        cc.context,
        &cc.binary,
        &cc.llvm_structs,
        ty,
        case,
    );

    // Save enum id:
    let int32_type =
        type_to_llvm_type(cc.context, &cc.llvm_structs, &Rc::new(I32_TYPE.to_string()));
    let zero = unsafe { llvm::core::LLVMConstInt(int32_type, 0, 0) };
    let case_id_index =
        unsafe { llvm::core::LLVMConstInt(int32_type, Gc::get_type_header_length(), 0) };
    let save_id_name = CString::new("case_id".to_string()).unwrap();
    unsafe {
        let ptr = llvm::core::LLVMBuildGEP(
            cc.builder,
            heap_ptr,
            vec![zero, case_id_index].as_mut_ptr(),
            2,
            save_id_name.as_ptr(),
        );
        llvm::core::LLVMBuildStore(cc.builder, id, ptr)
    };

    // Save NULL to all pointer fields:
    for f in pointer_indices {
        let field_ptr = CString::new("field_ptr".to_string()).unwrap();
        unsafe {
            let ptr = llvm::core::LLVMBuildGEP(
                cc.builder,
                heap_ptr,
                vec![zero, f].as_mut_ptr(),
                2,
                field_ptr.as_ptr(),
            );
            let ptr_type = llvm::core::LLVMGetElementType(llvm::core::LLVMTypeOf(ptr));
            llvm::core::LLVMBuildStore(cc.builder, llvm::core::LLVMConstNull(ptr_type), ptr);
        }
    }

    // Save fields:
    for (field, index) in fs.iter().zip(field_indices.into_iter()) {
        let field_ptr = CString::new("field_ptr".to_string()).unwrap();
        unsafe {
            let ptr = llvm::core::LLVMBuildGEP(
                cc.builder,
                heap_ptr,
                vec![zero, index].as_mut_ptr(),
                2,
                field_ptr.as_ptr(),
            );
            llvm::core::LLVMBuildStore(cc.builder, *field, ptr);
        }
    }

    heap_ptr
}

fn compute_params<'input, Gc: GC>(
    cc: &CodegenContext,
    current_func: *mut llvm::LLVMValue,
    vars: &mut HashMap<&'input str, *mut llvm::LLVMValue>,
    current_sp: *mut llvm::LLVMValue,
    params: &'input Vec<TypedExpr>,
) -> (Vec<Option<*mut llvm::LLVMValue>>, Vec<*mut llvm::LLVMValue>, *mut llvm::LLVMValue) {
    let mut computed_params: Vec<Option<*mut llvm::LLVMValue>> = Vec::new();
    let mut saved_params: Vec<*mut llvm::LLVMValue> = Vec::new();
    let mut sp = current_sp;
    for (i, param) in params.into_iter().enumerate() {
        let computed_param = build_expression::<Gc>(cc, current_func, vars, sp, &param);
        let type_first_char = param
            .expr_type
            .as_str()
            .chars()
            .next()
            .expect("Could not get first char of type");
        if type_first_char == '$' {
            // User defined types start with $ and need to be allocated on the
            // arena stack for the case that the next allocation leads to a
            // garbage collection
            let mut var_name = "$param$".to_string();
            var_name.push_str(i.to_string().as_str());
            let var_name_c = CString::new(var_name).unwrap();
            let (var, new_sp) = get_next_stack_element::<Gc>(sp, cc, &var_name_c, &param.expr_type);
            unsafe { llvm::core::LLVMBuildStore(cc.builder, computed_param, var) };
            saved_params.push(var);
            computed_params.push(None);
            sp = new_sp;
        } else {
            computed_params.push(Some(computed_param));
        }
    }
    (computed_params, saved_params, sp)
}

fn load_params(
    cc: &CodegenContext,
    computed_params: Vec<Option<*mut llvm::LLVMValue>>,
    mut stored_params: Vec<*mut llvm::LLVMValue>,
) -> Vec<*mut llvm::LLVMValue> {
    let mut loaded_params = Vec::new();
    for (i, param) in computed_params.into_iter().enumerate() {
        match param {
            None => {
                let mut var_name = "$param$".to_string();
                var_name.push_str(i.to_string().as_str());
                let var_name_c = CString::new(var_name).unwrap();
                loaded_params.push(unsafe {
                    llvm::core::LLVMBuildLoad(cc.builder, stored_params.remove(0), var_name_c.as_ptr())
                })
            },
            Some(p) => loaded_params.push(p),
        }
    }
    loaded_params
}
