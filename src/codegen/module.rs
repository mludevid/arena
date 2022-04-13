use crate::binary::Binary;
use crate::codegen::function;
use crate::codegen::CodegenContext;
use crate::types::create_structs;

use llvm_sys as llvm;
use std::ffi::CString;
use std::rc::Rc;

pub fn build_module<'input>(
    context: *mut llvm::LLVMContext,
    builder: *mut llvm::LLVMBuilder,
    binary: Binary<'input>,
) -> *mut llvm::LLVMModule {
    unsafe {
        let module_name = CString::new("ArenaBinary").unwrap();
        let llvm_module = llvm::core::LLVMModuleCreateWithName(module_name.as_ptr());
        let llvm_structs = create_structs(&binary, context);
        let cc = CodegenContext {
            binary,
            llvm_module,
            context,
            builder,
            llvm_structs,
        };
        let main_func = &cc
            .binary
            .functions
            .get(&Rc::new("main".to_string()))
            .expect("Could not find main codegen");
        let args = &main_func
            .args
            .iter()
            .map(|arg| Rc::clone(&arg.param_type))
            .collect::<Vec<_>>();
        function::init_build_function(
            &cc,
            "main",
            args,
            &main_func.ret_type,
            false,
            main_func,
            true,
        );
        llvm_module
    }
}
