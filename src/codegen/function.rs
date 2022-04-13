use crate::binary::BinFunction;
use crate::codegen::build_in::{get_build_in_func_call, get_linked_func_signature};
use crate::codegen::expression::build_expression;
use crate::codegen::CodegenContext;
use crate::types::{type_to_llvm_type, VOID_TYPE};

use llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

pub fn create_func_call(
    cc: &CodegenContext,
    func_id: &Rc<String>,
    computed_params: &mut Vec<*mut llvm::LLVMValue>,
) -> *mut llvm::LLVMValue {
    let func_name = CString::new(func_id.as_str()).unwrap();
    let func = unsafe { llvm::core::LLVMGetNamedFunction(cc.llvm_module, func_name.as_ptr()) };
    // if function already exists create call
    if func != std::ptr::null_mut() {
        append_func_call(cc, func, computed_params)
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
        let new_func = init_build_function(
            cc,
            func_id.as_str(),
            args,
            &bin_func.ret_type,
            false,
            bin_func,
            false,
        );
        append_func_call(cc, new_func, computed_params)
    } else {
        match get_build_in_func_call(cc, func_id, computed_params) {
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
                );
                append_func_call(cc, new_func, computed_params)
            }
        }
    }
}

fn append_func_call(
    cc: &CodegenContext,
    llvm_func: *mut llvm::LLVMValue,
    computed_params: &mut Vec<*mut llvm::LLVMValue>,
) -> *mut llvm::LLVMValue {
    unsafe {
        let args_len = computed_params.len().try_into().unwrap();
        if llvm::core::LLVMGetReturnType(llvm::core::LLVMGetReturnType(llvm::core::LLVMTypeOf(
            llvm_func,
        ))) == type_to_llvm_type(
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
        }
    }
}

pub fn init_build_function<'input>(
    cc: &'input CodegenContext,
    func_name: &str,
    args: &Vec<Rc<String>>,
    ret_type: &Rc<String>,
    is_var_arg: bool,
    func_def: &BinFunction,
    is_main: bool,
) -> *mut llvm::LLVMValue {
    let llvm_func = init_function(cc, func_name, args, ret_type, is_var_arg);
    build_function(cc, llvm_func, func_def, is_main);
    llvm_func
}

fn build_function<'input>(
    cc: &'input CodegenContext,
    llvm_func: *mut llvm::LLVMValue,
    function: &'input BinFunction,
    is_main: bool,
) {
    // If the function to create is not main it is being created because
    // another function building reached a func_call of this function
    // Therefore the builder has to be reset after this build to the old position
    let previous_block = if is_main {
        None
    } else {
        Some(unsafe { llvm::core::LLVMGetInsertBlock(cc.builder) })
    };
    create_entry(&cc, llvm_func);
    let mut vars: HashMap<&'input str, *mut llvm::LLVMValue> = HashMap::new();
    for (i, param) in function.args.iter().enumerate() {
        let id = unsafe { llvm::core::LLVMGetParam(llvm_func, i.try_into().unwrap()) };
        let var_name = CString::new(param.name).unwrap();
        let var = unsafe {
            llvm::core::LLVMBuildAlloca(
                cc.builder,
                type_to_llvm_type(cc.context, &cc.llvm_structs, &param.param_type),
                var_name.as_ptr(),
            )
        };
        unsafe { llvm::core::LLVMBuildStore(cc.builder, id, var) };
        vars.insert(param.name, var);
    }
    let res = build_expression(&cc, llvm_func, &mut vars, &function.body);
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
) -> *mut llvm::LLVMValue {
    unsafe {
        let args_len = args.len();
        let is_var_arg_int = if is_var_arg { 1 } else { 0 };
        let mut args_type: Vec<*mut llvm::LLVMType> = args
            .iter()
            .map(|arg| type_to_llvm_type(cc.context, &cc.llvm_structs, arg))
            .collect();
        let function_type = llvm::core::LLVMFunctionType(
            type_to_llvm_type(cc.context, &cc.llvm_structs, ret_type),
            args_type.as_mut_ptr(),
            args_len.try_into().unwrap(),
            is_var_arg_int,
        );
        let function_name = CString::new(name).unwrap();
        let func =
            llvm::core::LLVMAddFunction(cc.llvm_module, function_name.as_ptr(), function_type);

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
