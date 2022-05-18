pub mod build_in;
mod expression;
mod function;
pub mod garbage_collection;
mod module;

use llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

use crate::binary::Binary;
use crate::codegen::garbage_collection::GC;

pub fn codegen<Gc: GC>(binary: Binary, out_ll: &str, print_llvm_code: bool) {
    let (context, builder) = llvm_setup();

    let module = module::build_module::<Gc>(context, builder, binary);

    llvm_cleanup(context, module, builder, out_ll, print_llvm_code);
}

fn llvm_setup() -> (*mut llvm::LLVMContext, *mut llvm::LLVMBuilder) {
    unsafe {
        let context = llvm::core::LLVMContextCreate();
        let builder = llvm::core::LLVMCreateBuilderInContext(context);
        (context, builder)
    }
}

fn llvm_cleanup(
    context: *mut llvm::LLVMContext,
    module: *mut llvm::LLVMModule,
    builder: *mut llvm::LLVMBuilder,
    out_ll: &str,
    print_llvm_code: bool,
) {
    let stdout = CString::new("/dev/stdout").unwrap();
    let out_ll = CString::new(out_ll).unwrap();
    unsafe {
        if print_llvm_code {
            llvm::core::LLVMPrintModuleToFile(module, stdout.as_ptr(), ptr::null_mut());
        }
        llvm::core::LLVMPrintModuleToFile(module, out_ll.as_ptr(), ptr::null_mut());
        llvm::core::LLVMDisposeBuilder(builder);
        llvm::core::LLVMDisposeModule(module);
        llvm::core::LLVMContextDispose(context);
    }
}

pub struct CodegenContext<'input> {
    pub binary: Binary<'input>,
    pub llvm_module: *mut llvm::LLVMModule,
    pub context: *mut llvm::LLVMContext,
    pub builder: *mut llvm::LLVMBuilder,
    pub llvm_structs: HashMap<Rc<String>, *mut llvm::LLVMType>,
}
