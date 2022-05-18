use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

mod binary;
mod codegen;
mod input;
mod module;
mod parser;
mod type_check;
mod types;

use crate::codegen::garbage_collection::{Spill, ARC};
use crate::input::ClapGC;

fn main() {
    let (codes, cli) = input::input();

    let executable_name = std::env::current_dir().unwrap().join(match cli.o {
        None => PathBuf::from_str("out").unwrap(),
        Some(exe) => exe,
    });
    let ll_path = executable_name.with_extension("ll");
    let s_path = executable_name.with_extension("s");

    if cli.verbose {
        println!("CODE:");
        for (path, code) in &codes {
            println!("{}:\n{}\n", path.to_str().unwrap(), code);
        }
    }

    let asts = codes
        .iter()
        .map(|(path, code)| (path, parser::parse(code.as_str())))
        .collect::<HashMap<_, _>>();

    if cli.verbose || cli.print_ast {
        println!("AST:");
        for (path, ast) in &asts {
            println!("{}:\n{:#?}\n", path.to_str().unwrap(), ast);
        }
    }

    let (resolved_import_asts, main_module_id) = input::resolve_all_imports(asts, &cli.file_path);

    if cli.verbose {
        println!("Imports resolved AST:");
        for (prefix, ast) in &resolved_import_asts {
            println!("{}:\n{:#?}\n", prefix, ast);
        }
    }

    let typed_ast = type_check::type_check(&resolved_import_asts, &main_module_id)
        .expect("Type checking failed");

    if cli.verbose || cli.print_typed_ast {
        println!("TYPE-CHECKED AST:\n{:#?}", typed_ast);
    }

    match cli.gc {
        Some(ClapGC::Spill) => codegen::codegen::<Spill>(
            typed_ast,
            ll_path.to_str().unwrap(),
            cli.verbose || cli.print_llvm,
        ),
        None | Some(ClapGC::Arc) => codegen::codegen::<ARC>(
            typed_ast,
            ll_path.to_str().unwrap(),
            cli.verbose || cli.print_llvm,
        ),
    };

    let mut llc_o_arg = "-o=".to_string();
    llc_o_arg.push_str(s_path.to_str().unwrap());
    let llc_output = Command::new("llc")
        .arg("--relocation-model=pic")
        .arg(llc_o_arg.as_str())
        .arg(ll_path.to_str().unwrap())
        .output()
        .expect("Failed to execute llc");
    if !llc_output.status.success() {
        panic!(
            "llc failed with error code {:?}. Output:\n{}",
            llc_output.status.code(),
            String::from_utf8_lossy(&llc_output.stderr)
        )
    }
    let exe_path = std::env::current_exe().expect("Could not get executable path");
    let libarena_path = exe_path
        .parent()
        .expect("Could not get executable folder")
        .join("libarena.a");
    let gcc_output = Command::new("gcc")
        .arg("-Wextra")
        .arg(s_path.to_str().unwrap())
        .arg("-o")
        .arg(executable_name.to_str().unwrap())
        .arg(libarena_path.to_str().unwrap())
        .output()
        .expect("Failed to execute gcc");
    if !gcc_output.status.success() {
        panic!(
            "gcc failed with error code {:?}. Output:\n{}",
            gcc_output.status.code(),
            String::from_utf8_lossy(&gcc_output.stderr)
        )
    }

    if !cli.keep_temporaries {
        let rm_output_1 = Command::new("rm")
            .arg(ll_path.to_str().unwrap())
            .output()
            .expect("Failed to execute rm");
        if !rm_output_1.status.success() {
            panic!(
                "rm failed with error code {:?}. Output:\n{}",
                rm_output_1.status.code(),
                String::from_utf8_lossy(&rm_output_1.stderr)
            )
        }
        let rm_output_2 = Command::new("rm")
            .arg(s_path.to_str().unwrap())
            .output()
            .expect("Failed to execute rm");
        if !rm_output_2.status.success() {
            panic!(
                "rm failed with error code {:?}. Output:\n{}",
                rm_output_2.status.code(),
                String::from_utf8_lossy(&rm_output_2.stderr)
            )
        }
    }
}
