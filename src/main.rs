use std::collections::HashMap;
use std::process::Command;

mod binary;
mod codegen;
mod input;
mod module;
mod parser;
mod type_check;
mod types;

fn main() {
    let (codes, cli) = input::input();
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

    codegen::codegen(typed_ast, cli.verbose || cli.print_llvm);

    let llc_output = Command::new("llc")
        .arg("--relocation-model=pic")
        .arg("out.ll")
        .output()
        .expect("Failed to execute llc");
    if !llc_output.status.success() {
        panic!(
            "llc failed with error code {:?}. Output:\n{}",
            llc_output.status.code(),
            String::from_utf8_lossy(&llc_output.stderr)
        )
    }
    let gcc_output = Command::new("gcc")
        .arg("-Wextra")
        .arg("out.s")
        .arg("-o")
        .arg("out")
        .arg("libarena.a")
        .output()
        .expect("Failed to execute gcc");
    if !gcc_output.status.success() {
        panic!(
            "gcc failed with error code {:?}. Output:\n{}",
            gcc_output.status.code(),
            String::from_utf8_lossy(&gcc_output.stderr)
        )
    }
}
