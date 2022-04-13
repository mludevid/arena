use llvm_sys as llvm;
use std::ffi::CString;
use std::rc::Rc;

use crate::codegen::function::create_func_call;
use crate::codegen::CodegenContext;
use crate::types::{BOOL_TYPE, EXIT_TYPE, I32_TYPE, I64_TYPE, I8_PTR_TYPE, STRING_TYPE, VOID_TYPE};

macro_rules! enum_str {
    ($( #[$cfgs:meta] )*
        enum $name:ident {
        $($variant:ident),*,
    }) => {
        $( #[$cfgs] )*
        enum $name {
            $($variant),*
        }

        impl $name {
            fn as_str(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }

            fn from_str(s: &str) -> Self {
                match s {
                    $(stringify!($variant) => $name::$variant),*,
                    _ => panic!("ENUM VARIANT NOT FOUND: {}", s),
                }
            }
        }
    };
}

enum_str!(
    #[allow(non_camel_case_types)]
    enum BuildIn {
        print,
        printf,
        exit,
        malloc,
        eq_int,
        eq_bool,
        neq_bool,
        neq_int,
        lt_int,
        le_int,
        gt_int,
        ge_int,
        add_int,
        sub_int,
        mul_int,
        div_int,
        mod_int,
        neg_int,
        not_bool,
    }
);

pub fn get_build_in_signature(
    func_name: &str,
    arg_types: &Vec<Rc<String>>,
) -> Option<(Rc<String>, Rc<String>)> {
    // Returns (name, ret_type)
    match (
        func_name,
        &arg_types
            .iter()
            .map(|arg_type| arg_type.as_str())
            .collect::<Vec<_>>()[..],
    ) {
        ("print", [STRING_TYPE]) => Some((
            Rc::new(BuildIn::print.as_str().to_string()),
            Rc::new(VOID_TYPE.to_string()),
        )),
        ("exit", [I32_TYPE]) => Some((
            Rc::new(BuildIn::exit.as_str().to_string()),
            Rc::new(EXIT_TYPE.to_string()),
        )),
        ("eq", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::eq_int.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("eq", [BOOL_TYPE, BOOL_TYPE]) => Some((
            Rc::new(BuildIn::eq_bool.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("neq", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::neq_int.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("neq", [BOOL_TYPE, BOOL_TYPE]) => Some((
            Rc::new(BuildIn::neq_bool.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("lt", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::lt_int.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("le", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::le_int.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("gt", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::gt_int.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("ge", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::ge_int.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("add", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::add_int.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("sub", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::sub_int.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("mul", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::mul_int.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("div", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::div_int.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("mod", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::mod_int.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("neg", [I32_TYPE]) => Some((
            Rc::new(BuildIn::neg_int.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("not", [BOOL_TYPE]) => Some((
            Rc::new(BuildIn::not_bool.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        _ => None,
    }
}

pub fn get_build_in_func_call(
    cc: &CodegenContext,
    func_id: &Rc<String>,
    computed_params: &mut Vec<*mut llvm::LLVMValue>,
) -> Option<*mut llvm::LLVMValue> {
    match BuildIn::from_str(func_id.as_str()) {
        BuildIn::printf | BuildIn::exit | BuildIn::malloc => None,
        BuildIn::print => Some(create_func_call(
            cc,
            &Rc::new(BuildIn::printf.as_str().to_string()),
            computed_params,
        )),
        BuildIn::eq_int | BuildIn::eq_bool => unsafe {
            let name = CString::new("eqtmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntEQ,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::neq_int | BuildIn::neq_bool => unsafe {
            let name = CString::new("neqtmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntNE,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::lt_int => unsafe {
            let name = CString::new("lttmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSLT,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::le_int => unsafe {
            let name = CString::new("letmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSLE,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::gt_int => unsafe {
            let name = CString::new("gttmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSGT,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::ge_int => unsafe {
            let name = CString::new("getmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSGE,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::add_int => unsafe {
            let name = CString::new("addtmp").unwrap();
            Some(llvm::core::LLVMBuildAdd(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::sub_int => unsafe {
            let name = CString::new("subtmp").unwrap();
            Some(llvm::core::LLVMBuildSub(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::mul_int => unsafe {
            let name = CString::new("multmp").unwrap();
            Some(llvm::core::LLVMBuildMul(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::div_int => unsafe {
            let name = CString::new("divtmp").unwrap();
            Some(llvm::core::LLVMBuildSDiv(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::mod_int => unsafe {
            let name = CString::new("modtmp").unwrap();
            Some(llvm::core::LLVMBuildSRem(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::neg_int => unsafe {
            let name = CString::new("negtmp").unwrap();
            Some(llvm::core::LLVMBuildNeg(
                cc.builder,
                computed_params[0],
                name.as_ptr(),
            ))
        },
        BuildIn::not_bool => unsafe {
            let name = CString::new("nottmp").unwrap();
            Some(llvm::core::LLVMBuildNot(
                cc.builder,
                computed_params[0],
                name.as_ptr(),
            ))
        },
    }
}

pub fn get_linked_func_signature(func_id: &Rc<String>) -> (Vec<&'static str>, &'static str, bool) {
    // returns arg types, ret type, is var arg
    match BuildIn::from_str(func_id.as_str()) {
        BuildIn::printf => (vec![STRING_TYPE], I32_TYPE, true),
        BuildIn::exit => (vec![I32_TYPE], EXIT_TYPE, false),
        BuildIn::malloc => (vec![I64_TYPE], I8_PTR_TYPE, false),
        BuildIn::print
        | BuildIn::eq_int
        | BuildIn::eq_bool
        | BuildIn::neq_bool
        | BuildIn::neq_int
        | BuildIn::lt_int
        | BuildIn::le_int
        | BuildIn::gt_int
        | BuildIn::ge_int
        | BuildIn::add_int
        | BuildIn::sub_int
        | BuildIn::mul_int
        | BuildIn::div_int
        | BuildIn::mod_int
        | BuildIn::neg_int
        | BuildIn::not_bool => unreachable!("{} is not a dynamically linked function", func_id),
    }
}
