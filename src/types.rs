use llvm_sys as llvm;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

use crate::binary::Binary;

pub const U8_TYPE: &'static str = "u8";
pub const I32_TYPE: &'static str = "i32";
pub const BOOL_TYPE: &'static str = "bool";
pub const STR_TYPE: &'static str = "str";
pub const VOID_TYPE: &'static str = "void";
pub const VOID_PTR_TYPE: &'static str = "i8*"; // LLVM does not support void*
pub const EXIT_TYPE: &'static str = "$exit$";

pub fn type_to_llvm_type(
    context: *mut llvm::LLVMContext,
    llvm_structs: &HashMap<Rc<String>, *mut llvm::LLVMType>,
    ty: &Rc<String>,
) -> *mut llvm::LLVMType {
    match ty.as_str() {
        "i8*" => unsafe {
            llvm::core::LLVMPointerType(llvm::core::LLVMInt8TypeInContext(context), 0)
        },
        "u8" => unsafe { llvm::core::LLVMInt8TypeInContext(context) },
        "i32" => unsafe { llvm::core::LLVMInt32TypeInContext(context) },
        "i64" => unsafe { llvm::core::LLVMInt64TypeInContext(context) },
        "bool" => unsafe { llvm::core::LLVMInt1TypeInContext(context) },
        "str" => {
            let int8_type = unsafe { llvm::core::LLVMInt8TypeInContext(context) };
            unsafe { llvm::core::LLVMPointerType(int8_type, 0) }
        }
        "void" => unsafe { llvm::core::LLVMVoidTypeInContext(context) },
        "$exit$" => unsafe { llvm::core::LLVMVoidTypeInContext(context) },
        _ => unsafe {
            llvm::core::LLVMPointerType(
                *llvm_structs
                    .get(ty)
                    .unwrap_or_else(|| panic!("Could not find llvm struct {}", ty)),
                0,
            )
        },
    }
}

pub fn create_structs(
    binary: &Binary,
    context: *mut llvm::LLVMContext,
) -> HashMap<Rc<String>, *mut llvm::LLVMType> {
    let mut ret = HashMap::new();
    for (name, _) in binary.types.iter() {
        let type_name = CString::new(name.as_str()).unwrap();
        let llvm_struct = unsafe { llvm::core::LLVMStructCreateNamed(context, type_name.as_ptr()) };
        ret.insert(Rc::clone(&name), llvm_struct);
    }

    for (name, t) in binary.types.iter() {
        let llvm_struct = *ret.get(name).unwrap();
        let mut fields = vec![unsafe { llvm::core::LLVMInt32TypeInContext(context) }];
        for case in t.cases.iter() {
            for f in case.fields.iter() {
                fields.push(type_to_llvm_type(context, &ret, f));
            }
        }
        let fields_len = fields.len().try_into().unwrap();
        unsafe { llvm::core::LLVMStructSetBody(llvm_struct, fields.as_mut_ptr(), fields_len, 0) };
    }
    ret
}

pub fn get_case_id_indices(
    context: *mut llvm::LLVMContext,
    binary: &Binary,
    llvm_structs: &HashMap<Rc<String>, *mut llvm::LLVMType>,
    ty: &Rc<String>,
    case: &str,
) -> (*mut llvm::LLVMValue, Vec<*mut llvm::LLVMValue>) {
    let type_def = binary
        .types
        .get(ty)
        .expect("Could not find type def in binary");
    let (mut case_index, mut case_field_indices): (Option<_>, Option<Vec<_>>) = (None, None);
    let mut field_count: u64 = 1;
    for (i, c) in type_def.cases.iter().enumerate() {
        if c.name == case {
            case_index = Some(i.try_into().unwrap());
            let case_field_count: u64 = c.fields.len().try_into().unwrap();
            case_field_indices = Some((field_count..(field_count + case_field_count)).collect());
            break;
        } else {
            let case_field_count: u64 = c.fields.len().try_into().unwrap();
            field_count += case_field_count;
        }
    }
    let int32_type = type_to_llvm_type(context, llvm_structs, &Rc::new(I32_TYPE.to_string()));
    (
        unsafe {
            llvm::core::LLVMConstInt(
                int32_type,
                case_index.expect("Internal error: Could not find case index"),
                0,
            )
        },
        case_field_indices
            .unwrap()
            .into_iter()
            .map(|i| unsafe { llvm::core::LLVMConstInt(int32_type, i, 0) })
            .collect::<Vec<_>>(),
    )
}
