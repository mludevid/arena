use llvm_sys as llvm;
use std::ffi::CString;
use std::rc::Rc;

use crate::codegen::function::create_func_call;
use crate::codegen::CodegenContext;
use crate::types::{
    BOOL_TYPE, EXIT_TYPE, I32_TYPE, I64_TYPE, I8_PTR_TYPE, STR_TYPE, U8_TYPE, VOID_TYPE,
};

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
        print_str,
        print_u8,
        print_i32,
        printf,
        char_at,
        exit,
        malloc,
        eq_i32,
        eq_u8,
        eq_bool,
        neq_bool,
        neq_i32,
        neq_u8,
        lt_i32,
        lt_u8,
        le_i32,
        le_u8,
        gt_i32,
        gt_u8,
        ge_i32,
        ge_u8,
        add_i32,
        add_u8,
        sub_i32,
        sub_u8,
        mul_i32,
        mul_u8,
        div_i32,
        div_u8,
        mod_i32,
        mod_u8,
        neg_i32,
        neg_u8,
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
        ("print", [STR_TYPE]) => Some((
            Rc::new(BuildIn::print_str.as_str().to_string()),
            Rc::new(VOID_TYPE.to_string()),
        )),
        ("print", [U8_TYPE]) => Some((
            Rc::new(BuildIn::print_u8.as_str().to_string()),
            Rc::new(VOID_TYPE.to_string()),
        )),
        ("print", [I32_TYPE]) => Some((
            Rc::new(BuildIn::print_i32.as_str().to_string()),
            Rc::new(VOID_TYPE.to_string()),
        )),
        ("char_at", [STR_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::char_at.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("exit", [I32_TYPE]) => Some((
            Rc::new(BuildIn::exit.as_str().to_string()),
            Rc::new(EXIT_TYPE.to_string()),
        )),
        ("eq", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::eq_i32.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("eq", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::eq_u8.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("eq", [BOOL_TYPE, BOOL_TYPE]) => Some((
            Rc::new(BuildIn::eq_bool.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("neq", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::neq_i32.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("neq", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::neq_u8.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("neq", [BOOL_TYPE, BOOL_TYPE]) => Some((
            Rc::new(BuildIn::neq_bool.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("lt", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::lt_u8.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("le", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::le_u8.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("gt", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::gt_u8.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("ge", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::ge_u8.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("add", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::add_u8.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("sub", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::sub_u8.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("mul", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::mul_u8.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("div", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::div_u8.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("mod", [U8_TYPE, U8_TYPE]) => Some((
            Rc::new(BuildIn::mod_u8.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("neg", [U8_TYPE]) => Some((
            Rc::new(BuildIn::neg_u8.as_str().to_string()),
            Rc::new(U8_TYPE.to_string()),
        )),
        ("lt", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::lt_i32.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("le", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::le_i32.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("gt", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::gt_i32.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("ge", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::ge_i32.as_str().to_string()),
            Rc::new(BOOL_TYPE.to_string()),
        )),
        ("add", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::add_i32.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("sub", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::sub_i32.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("mul", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::mul_i32.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("div", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::div_i32.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("mod", [I32_TYPE, I32_TYPE]) => Some((
            Rc::new(BuildIn::mod_i32.as_str().to_string()),
            Rc::new(I32_TYPE.to_string()),
        )),
        ("neg", [I32_TYPE]) => Some((
            Rc::new(BuildIn::neg_i32.as_str().to_string()),
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
        BuildIn::print_str => Some(create_func_call(
            cc,
            &Rc::new(BuildIn::printf.as_str().to_string()),
            computed_params,
        )),
        BuildIn::print_i32 => {
            let c_str = CString::new("%d").unwrap();
            let name = CString::new(".str").unwrap();
            let s = unsafe {
                llvm::core::LLVMBuildGlobalStringPtr(cc.builder, c_str.as_ptr(), name.as_ptr())
            };
            let mut params = vec![s];
            params.append(computed_params);
            Some(create_func_call(
                cc,
                &Rc::new(BuildIn::printf.as_str().to_string()),
                &mut params,
            ))
        }
        BuildIn::print_u8 => {
            let c_str = CString::new("%c").unwrap();
            let name = CString::new(".str").unwrap();
            let s = unsafe {
                llvm::core::LLVMBuildGlobalStringPtr(cc.builder, c_str.as_ptr(), name.as_ptr())
            };
            let mut params = vec![s];
            params.append(computed_params);
            Some(create_func_call(
                cc,
                &Rc::new(BuildIn::printf.as_str().to_string()),
                &mut params,
            ))
        }
        BuildIn::char_at => unsafe {
            let mut indices = vec![computed_params[1]];
            let char_ptr_name = CString::new("char_at_ptr").unwrap();
            let char_at_name = CString::new("char_at").unwrap();
            let char_ptr = llvm::core::LLVMBuildGEP(
                cc.builder,
                computed_params[0],
                indices.as_mut_ptr(),
                indices.len().try_into().unwrap(),
                char_ptr_name.as_ptr(),
            );
            Some(llvm::core::LLVMBuildLoad(
                cc.builder,
                char_ptr,
                char_at_name.as_ptr(),
            ))
        },
        BuildIn::eq_u8 | BuildIn::eq_i32 | BuildIn::eq_bool => unsafe {
            let name = CString::new("eqtmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntEQ,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::neq_u8 | BuildIn::neq_i32 | BuildIn::neq_bool => unsafe {
            let name = CString::new("neqtmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntNE,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::lt_u8 | BuildIn::lt_i32 => unsafe {
            let name = CString::new("lttmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSLT,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::le_u8 | BuildIn::le_i32 => unsafe {
            let name = CString::new("letmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSLE,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::gt_u8 | BuildIn::gt_i32 => unsafe {
            let name = CString::new("gttmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSGT,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::ge_u8 | BuildIn::ge_i32 => unsafe {
            let name = CString::new("getmp").unwrap();
            Some(llvm::core::LLVMBuildICmp(
                cc.builder,
                llvm::LLVMIntPredicate::LLVMIntSGE,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::add_u8 | BuildIn::add_i32 => unsafe {
            let name = CString::new("addtmp").unwrap();
            Some(llvm::core::LLVMBuildAdd(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::sub_u8 | BuildIn::sub_i32 => unsafe {
            let name = CString::new("subtmp").unwrap();
            Some(llvm::core::LLVMBuildSub(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::mul_u8 | BuildIn::mul_i32 => unsafe {
            let name = CString::new("multmp").unwrap();
            Some(llvm::core::LLVMBuildMul(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::div_u8 | BuildIn::div_i32 => unsafe {
            let name = CString::new("divtmp").unwrap();
            Some(llvm::core::LLVMBuildSDiv(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::mod_u8 | BuildIn::mod_i32 => unsafe {
            let name = CString::new("modtmp").unwrap();
            Some(llvm::core::LLVMBuildSRem(
                cc.builder,
                computed_params[0],
                computed_params[1],
                name.as_ptr(),
            ))
        },
        BuildIn::neg_u8 | BuildIn::neg_i32 => unsafe {
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
        BuildIn::printf => (vec![STR_TYPE], I32_TYPE, true),
        BuildIn::exit => (vec![I32_TYPE], EXIT_TYPE, false),
        BuildIn::malloc => (vec![I64_TYPE], I8_PTR_TYPE, false),
        BuildIn::char_at
        | BuildIn::print_str
        | BuildIn::print_u8
        | BuildIn::print_i32
        | BuildIn::eq_u8
        | BuildIn::eq_i32
        | BuildIn::eq_bool
        | BuildIn::neq_bool
        | BuildIn::neq_u8
        | BuildIn::lt_u8
        | BuildIn::le_u8
        | BuildIn::gt_u8
        | BuildIn::ge_u8
        | BuildIn::add_u8
        | BuildIn::sub_u8
        | BuildIn::mul_u8
        | BuildIn::div_u8
        | BuildIn::mod_u8
        | BuildIn::neg_u8
        | BuildIn::neq_i32
        | BuildIn::lt_i32
        | BuildIn::le_i32
        | BuildIn::gt_i32
        | BuildIn::ge_i32
        | BuildIn::add_i32
        | BuildIn::sub_i32
        | BuildIn::mul_i32
        | BuildIn::div_i32
        | BuildIn::mod_i32
        | BuildIn::neg_i32
        | BuildIn::not_bool => unreachable!("{} is not a dynamically linked function", func_id),
    }
}
