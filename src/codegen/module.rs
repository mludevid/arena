use crate::binary::Binary;
use crate::codegen::function;
use crate::codegen::garbage_collection::GC;
use crate::codegen::CodegenContext;
use crate::types::create_structs;

use llvm_sys as llvm;
use std::ffi::CString;
use std::rc::Rc;

pub fn build_module<'input, Gc: GC>(
    context: *mut llvm::LLVMContext,
    builder: *mut llvm::LLVMBuilder,
    binary: Binary<'input>,
    profiling_frequency: u64,
) -> *mut llvm::LLVMModule {
    unsafe {
        let module_name = CString::new("ArenaBinary").unwrap();
        let llvm_module = llvm::core::LLVMModuleCreateWithName(module_name.as_ptr());
        let llvm_structs = create_structs::<Gc>(&binary, context);
        let profiling_frequency = llvm::core::LLVMConstInt(llvm::core::LLVMInt64TypeInContext(context), profiling_frequency, 0);
        let cc = CodegenContext {
            binary,
            llvm_module,
            context,
            builder,
            llvm_structs,
            profiling_frequency,
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
        function::init_build_function::<Gc>(
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
