use crate::binary::BinFunction;
use crate::codegen::build_in::BuildIn::{close_stack, init_stack, stack_alloc};
use crate::codegen::build_in::{get_build_in_func_call, get_linked_func_signature};
use crate::codegen::expression::build_expression;
use crate::codegen::garbage_collection::GC;
use crate::codegen::CodegenContext;
use crate::types::{type_to_llvm_type, VOID_PTR_TYPE, VOID_TYPE};

use llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

pub fn create_func_call<Gc: GC>(
    cc: &CodegenContext,
    func_id: &Rc<String>,
    computed_params: &mut Vec<*mut llvm::LLVMValue>,
    sp: *mut llvm::LLVMValue,
) -> *mut llvm::LLVMValue {
    let func_name = CString::new(func_id.as_str()).unwrap();
    let func = unsafe { llvm::core::LLVMGetNamedFunction(cc.llvm_module, func_name.as_ptr()) };
    // if function already exists create call
    if func != std::ptr::null_mut() {
        append_func_call(cc, func, computed_params, sp)
    } else if cc.binary.functions.contains_key(func_id) {
        // else if function does not exists but is a function of the binary: create it and call it
        let bin_func = cc
            .binary
            .functions
            .get(func_id)
            .unwrap_or_else(|| panic!("Codegen: Could not find {}", func_id.as_str()));
        let args = &bin_func
            .args
            .iter()
            .map(|arg| Rc::clone(&arg.param_type))
            .collect::<Vec<_>>();
        let new_func = init_build_function::<Gc>(
            cc,
            func_id.as_str(),
            args,
            &bin_func.ret_type,
            false,
            bin_func,
            false,
        );
        append_func_call(cc, new_func, computed_params, sp)
    } else {
        match get_build_in_func_call::<Gc>(cc, func_id, computed_params, sp) {
            Some(val) => val,
            None => {
                // This *should* be a function that can be dynamically linked and is not yet
                // created
                let (arg_types, ret_type, is_var_arg) = get_linked_func_signature(func_id);
                let new_func = init_function(
                    cc,
                    func_id.as_str(),
                    &arg_types
                        .into_iter()
                        .map(|arg| Rc::new(arg.to_string()))
                        .collect(),
                    &Rc::new(ret_type.to_string()),
                    is_var_arg,
                    true,
                    false,
                );
                append_func_call(cc, new_func, computed_params, sp)
            }
        }
    }
}

fn append_func_call(
    cc: &CodegenContext,
    llvm_func: *mut llvm::LLVMValue,
    computed_params: &mut Vec<*mut llvm::LLVMValue>,
    sp: *mut llvm::LLVMValue,
) -> *mut llvm::LLVMValue {
    unsafe {
        let args_len: u32 = computed_params.len().try_into().unwrap();
        let func_args_len = llvm::core::LLVMCountParams(llvm_func);
        if args_len + 1 == func_args_len {
            computed_params.insert(0, sp);
        }

        // recompute args_len because it could have changed
        let args_len = computed_params.len().try_into().unwrap();

        let func_call = if llvm::core::LLVMGetReturnType(llvm::core::LLVMGetReturnType(
            llvm::core::LLVMTypeOf(llvm_func),
        )) == type_to_llvm_type(
            cc.context,
            &cc.llvm_structs,
            &Rc::new(VOID_TYPE.to_string()),
        ) {
            let call_name = CString::new("").unwrap();
            llvm::core::LLVMBuildCall(
                cc.builder,
                llvm_func,
                computed_params.as_mut_ptr(),
                args_len,
                call_name.as_ptr(),
            )
        } else {
            let call_name = CString::new("callret").unwrap();
            llvm::core::LLVMBuildCall(
                cc.builder,
                llvm_func,
                computed_params.as_mut_ptr(),
                args_len,
                call_name.as_ptr(),
            )
        };
        llvm::core::LLVMSetInstructionCallConv(
            func_call,
            llvm::core::LLVMGetFunctionCallConv(llvm_func),
        );
        func_call
    }
}

pub fn init_build_function<'input, Gc: GC>(
    cc: &'input CodegenContext,
    func_name: &str,
    args: &Vec<Rc<String>>,
    ret_type: &Rc<String>,
    is_var_arg: bool,
    func_def: &BinFunction,
    is_main: bool,
) -> *mut llvm::LLVMValue {
    let llvm_func = init_function(cc, func_name, args, ret_type, is_var_arg, false, is_main);
    build_function::<Gc>(cc, llvm_func, func_def, is_main);
    llvm_func
}

fn build_function<'input, Gc: GC>(
    cc: &'input CodegenContext,
    llvm_func: *mut llvm::LLVMValue,
    function: &'input BinFunction,
    is_main: bool,
) {
    // If the function to create is not main it is being created because
    // another function building reached a func_call of this function
    // Therefore the builder has to be reset after this build to the old position
    // If the function to create is not main the first argument is the SP of the
    // arena stack therefore the indices have to be offset by one
    let (previous_block, parameter_index_offset) = if is_main {
        (None, 0)
    } else {
        (
            Some(unsafe { llvm::core::LLVMGetInsertBlock(cc.builder) }),
            1,
        )
    };
    create_entry(&cc, llvm_func);
    let mut sp = if is_main {
        Gc::init_heap(cc);
        create_func_call::<Gc>(
            cc,
            &Rc::new(init_stack.as_str().to_string()),
            &mut vec![cc.profiling_frequency],
            std::ptr::null_mut(),
        )
    } else {
        unsafe { llvm::core::LLVMGetParam(llvm_func, 0) }
    };
    let mut vars: HashMap<&'input str, *mut llvm::LLVMValue> = HashMap::new();
    for (i, param) in function.args.iter().enumerate() {
        let id = unsafe {
            llvm::core::LLVMGetParam(llvm_func, (i + parameter_index_offset).try_into().unwrap())
        };
        let var_name = CString::new(param.name).unwrap();
        let type_first_char = param
            .param_type
            .as_str()
            .chars()
            .next()
            .expect("Could not get first char of param type");
        let (var, new_sp) = if type_first_char == '$' {
            // User defined types start with $ and go on the arena stack
            let sp_ret = create_func_call::<Gc>(
                cc,
                &Rc::new(stack_alloc.as_str().to_string()),
                &mut vec![sp],
                sp,
            );
            unsafe {
                (
                    llvm::core::LLVMBuildBitCast(
                        cc.builder,
                        sp_ret,
                        llvm::core::LLVMPointerType(
                            type_to_llvm_type(cc.context, &cc.llvm_structs, &param.param_type),
                            0,
                        ),
                        var_name.as_ptr(),
                    ),
                    sp_ret,
                )
            }
        } else {
            // Other types go on the regular stack
            unsafe {
                (
                    llvm::core::LLVMBuildAlloca(
                        cc.builder,
                        type_to_llvm_type(cc.context, &cc.llvm_structs, &param.param_type),
                        var_name.as_ptr(),
                    ),
                    sp,
                )
            }
        };
        sp = new_sp;
        unsafe { llvm::core::LLVMBuildStore(cc.builder, id, var) };
        vars.insert(param.name, var);
    }
    let res = build_expression::<Gc>(&cc, llvm_func, &mut vars, sp, &function.body);
    for (i, param) in function.args.iter().enumerate() {
        let id = unsafe {
            llvm::core::LLVMGetParam(llvm_func, (i + parameter_index_offset).try_into().unwrap())
        };
        let type_first_char = param
            .param_type
            .as_str()
            .chars()
            .next()
            .expect("Could not get first char of param type");
        if type_first_char == '$' {
            // User defined types start with $ and go on the arena stack
            Gc::type_ptr_drop(cc, id, sp);
        }
    }
    if is_main {
        create_func_call::<Gc>(
            cc,
            &Rc::new(close_stack.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
        Gc::close_heap(cc);
    }
    if function.ret_type.as_str() == VOID_TYPE {
        // If the return type of the function is void it should not return the
        // last value, it should drop it and return void instead
        unsafe { llvm::core::LLVMBuildRetVoid(cc.builder) };
    } else {
        unsafe { llvm::core::LLVMBuildRet(cc.builder, res) };
    }
    match previous_block {
        None => (),
        Some(bb) => unsafe { llvm::core::LLVMPositionBuilderAtEnd(cc.builder, bb) },
    }
}

fn init_function(
    cc: &CodegenContext,
    name: &str,
    args: &Vec<Rc<String>>,
    ret_type: &Rc<String>,
    is_var_arg: bool,
    is_build_in: bool,
    is_main: bool,
) -> *mut llvm::LLVMValue {
    unsafe {
        let is_var_arg_int = if is_var_arg { 1 } else { 0 };
        let mut args_type: Vec<*mut llvm::LLVMType> = args
            .iter()
            .map(|arg| type_to_llvm_type(cc.context, &cc.llvm_structs, arg))
            .collect();
        if !is_build_in && !is_main {
            args_type.insert(
                0,
                type_to_llvm_type(
                    cc.context,
                    &cc.llvm_structs,
                    &Rc::new(VOID_PTR_TYPE.to_string()),
                ),
            );
        }
        let args_len = args_type.len();
        let function_type = llvm::core::LLVMFunctionType(
            type_to_llvm_type(cc.context, &cc.llvm_structs, ret_type),
            args_type.as_mut_ptr(),
            args_len.try_into().unwrap(),
            is_var_arg_int,
        );
        let function_name = CString::new(name).unwrap();
        let func =
            llvm::core::LLVMAddFunction(cc.llvm_module, function_name.as_ptr(), function_type);

        if !is_build_in {
            // llvm::LLVMCallConv::LLVMFastCallConv => 8
            // llvm::core::LLVMSetFunctionCallConv(func, 8);
        }

        // TODO: Learn more about Attributes and decide which ones to add
        let no_inline_name = CString::new("noinline").unwrap();
        let no_inline = llvm::core::LLVMGetEnumAttributeKindForName(no_inline_name.as_ptr(), 8);
        let attr1 = llvm::core::LLVMCreateEnumAttribute(cc.context, no_inline, 0);
        let optnone_name = CString::new("optnone").unwrap();
        let optnone = llvm::core::LLVMGetEnumAttributeKindForName(optnone_name.as_ptr(), 7);
        let attr2 = llvm::core::LLVMCreateEnumAttribute(cc.context, optnone, 0);

        llvm::core::LLVMAddAttributeAtIndex(func, llvm::LLVMAttributeFunctionIndex, attr1);
        llvm::core::LLVMAddAttributeAtIndex(func, llvm::LLVMAttributeFunctionIndex, attr2);
        func
    }
}

fn create_entry(cc: &CodegenContext, func: *mut llvm::LLVMValue) {
    let entry_name = CString::new("entry").unwrap();
    unsafe {
        let bb = llvm::core::LLVMAppendBasicBlockInContext(cc.context, func, entry_name.as_ptr());
        llvm::core::LLVMPositionBuilderAtEnd(cc.builder, bb);
    }
}
