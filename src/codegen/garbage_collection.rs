use llvm_sys as llvm;
use std::ffi::CString;
use std::rc::Rc;

use crate::codegen::build_in::BuildIn;
use crate::codegen::function::create_func_call;
use crate::codegen::CodegenContext;
use crate::types::{type_to_llvm_type, VOID_PTR_TYPE};

pub trait GC {
    fn get_type_header(context: *mut llvm::LLVMContext) -> Vec<*mut llvm::LLVMType>;

    fn get_type_header_length() -> u64;

    fn init_header(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, size: *mut llvm::LLVMValue);

    fn init_heap(cc: &CodegenContext);

    fn close_heap(cc: &CodegenContext);

    fn type_allocation(
        cc: &CodegenContext,
        size: *mut llvm::LLVMValue,
        current_sp: *mut llvm::LLVMValue,
    ) -> *mut llvm::LLVMValue;

    fn type_ptr_access(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue);

    fn type_ptr_drop(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue);
}

pub struct Spill {}

impl GC for Spill {
    #[allow(unused_variables)]
    fn get_type_header(context: *mut llvm::LLVMContext) -> Vec<*mut llvm::LLVMType> {
        Vec::new()
    }

    fn get_type_header_length() -> u64 {
        0
    }

    #[allow(unused_variables)]
    fn init_header(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, size: *mut llvm::LLVMValue) {}

    #[allow(unused_variables)]
    fn init_heap(cc: &CodegenContext) {
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::init_heap.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
    }

    fn close_heap(cc: &CodegenContext) {
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::close_heap.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
    }

    fn type_allocation(
        cc: &CodegenContext,
        size: *mut llvm::LLVMValue,
        current_sp: *mut llvm::LLVMValue,
    ) -> *mut llvm::LLVMValue {
        create_func_call::<Self>(
            cc,
            &Rc::new("type_alloc".to_string()),
            &mut vec![size, current_sp],
            current_sp,
        )
    }

    #[allow(unused_variables)]
    fn type_ptr_access(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue) {}

    #[allow(unused_variables)]
    fn type_ptr_drop(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue) {}
}

pub struct ARC {}

impl GC for ARC {
    fn get_type_header(context: *mut llvm::LLVMContext) -> Vec<*mut llvm::LLVMType> {
        vec![unsafe { llvm::core::LLVMInt32TypeInContext(context) }]
    }

    fn get_type_header_length() -> u64 {
        1
    }

    #[allow(unused_variables)]
    fn init_header(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, size: *mut llvm::LLVMValue) {
        let int32_type = unsafe { llvm::core::LLVMInt32TypeInContext(cc.context) };
        let zero = unsafe { llvm::core::LLVMConstInt(int32_type, 0, 0) };
        let one = unsafe { llvm::core::LLVMConstInt(int32_type, 1, 0) };
        let arc_header_name = CString::new("arc_header".to_string()).unwrap();
        let arc_header_ptr = unsafe {
            llvm::core::LLVMBuildGEP(
                cc.builder,
                ptr,
                vec![zero, zero].as_mut_ptr(),
                2,
                arc_header_name.as_ptr(),
            )
        };
        unsafe { llvm::core::LLVMBuildStore(cc.builder, one, arc_header_ptr) };
    }

    #[allow(unused_variables)]
    fn init_heap(cc: &CodegenContext) {
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::init_heap.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
    }

    fn close_heap(cc: &CodegenContext) {
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::close_heap.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
    }

    fn type_allocation(
        cc: &CodegenContext,
        size: *mut llvm::LLVMValue,
        current_sp: *mut llvm::LLVMValue,
    ) -> *mut llvm::LLVMValue {
        create_func_call::<Self>(
            cc,
            &Rc::new("type_alloc".to_string()),
            &mut vec![size, current_sp],
            current_sp,
        )
    }

    fn type_ptr_access(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue) {
        let arc_count_name = CString::new("arc_count".to_string()).unwrap();
        let heap_ptr = unsafe {
            llvm::core::LLVMBuildBitCast(
                cc.builder,
                ptr,
                type_to_llvm_type(
                    cc.context,
                    &cc.llvm_structs,
                    &Rc::new(VOID_PTR_TYPE.to_string()),
                ),
                arc_count_name.as_ptr(),
            )
        };
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::arc_ptr_access.as_str().to_string()),
            &mut vec![heap_ptr, current_sp],
            std::ptr::null_mut(),
        );
    }

    fn type_ptr_drop(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue) {
        let arc_count_name = CString::new("arc_count".to_string()).unwrap();
        let heap_ptr = unsafe {
            llvm::core::LLVMBuildBitCast(
                cc.builder,
                ptr,
                type_to_llvm_type(
                    cc.context,
                    &cc.llvm_structs,
                    &Rc::new(VOID_PTR_TYPE.to_string()),
                ),
                arc_count_name.as_ptr(),
            )
        };
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::arc_drop_ptr.as_str().to_string()),
            &mut vec![heap_ptr, current_sp],
            std::ptr::null_mut(),
        );
    }
}

pub struct TGC {}

impl GC for TGC {
    fn get_type_header(context: *mut llvm::LLVMContext) -> Vec<*mut llvm::LLVMType> {
        vec![unsafe { llvm::core::LLVMInt32TypeInContext(context) }]
    }

    fn get_type_header_length() -> u64 {
        1
    }

    #[allow(unused_variables)]
    fn init_header(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, size: *mut llvm::LLVMValue) {
        let int32_type = unsafe { llvm::core::LLVMInt32TypeInContext(cc.context) };
        let zero = unsafe { llvm::core::LLVMConstInt(int32_type, 0, 0) };
        let tgc_header_name = CString::new("tgc_header".to_string()).unwrap();
        let tgc_header_ptr = unsafe {
            llvm::core::LLVMBuildGEP(
                cc.builder,
                ptr,
                vec![zero, zero].as_mut_ptr(),
                2,
                tgc_header_name.as_ptr(),
            )
        };
        let cast_name = CString::new("OBJ_SIZE".to_string()).unwrap();
        let size_u32 = unsafe {
            llvm::core::LLVMBuildIntCast(cc.builder, size, int32_type, cast_name.as_ptr())
        };
        unsafe { llvm::core::LLVMBuildStore(cc.builder, size_u32, tgc_header_ptr) };
    }

    #[allow(unused_variables)]
    fn init_heap(cc: &CodegenContext) {
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::tgc_init_heap.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
    }

    fn close_heap(cc: &CodegenContext) {
        create_func_call::<Self>(
            cc,
            &Rc::new(BuildIn::tgc_close_heap.as_str().to_string()),
            &mut Vec::new(),
            std::ptr::null_mut(),
        );
    }

    fn type_allocation(
        cc: &CodegenContext,
        size: *mut llvm::LLVMValue,
        current_sp: *mut llvm::LLVMValue,
    ) -> *mut llvm::LLVMValue {
        create_func_call::<Self>(
            cc,
            &Rc::new("tgc_type_alloc".to_string()),
            &mut vec![size, current_sp],
            current_sp,
        )
    }

    #[allow(unused_variables)]
    fn type_ptr_access(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue) {}

    #[allow(unused_variables)]
    fn type_ptr_drop(cc: &CodegenContext, ptr: *mut llvm::LLVMValue, current_sp: *mut llvm::LLVMValue) {}
}
