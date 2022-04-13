use clap::Parser;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::rc::Rc;

use crate::module::Module;
use crate::parser::parse_imports;

#[derive(Parser)]
#[clap(author, version, about)] // TODO: Add author and about to toml
pub struct Cli {
    /// Path of code to be compiled
    #[clap(parse(from_os_str), value_name = "FILE")]
    pub file_path: PathBuf,

    /// Print Code, AST and LLVM Code
    #[clap(short, long)]
    pub verbose: bool,

    /// Print AST
    #[clap(long)]
    pub print_ast: bool,

    /// Print Typed AST
    #[clap(long)]
    pub print_typed_ast: bool,

    /// Print LLVM Code
    #[clap(long)]
    pub print_llvm: bool,
}

pub fn input() -> (HashMap<PathBuf, String>, Cli) {
    let mut cli = Cli::parse();
    cli.file_path = fs::canonicalize(cli.file_path).expect("Incorrect Path");
    let mut files = HashMap::new();
    import_files(&mut files, vec![cli.file_path.clone()]);
    (files, cli)
}

fn import_files(files: &mut HashMap<PathBuf, String>, mut import_queue: Vec<PathBuf>) {
    let tail = import_queue.pop();
    match tail {
        None => (),
        Some(path) => {
            let code = fs::read_to_string(&path).expect("Something went wrong reading the file");
            let imports = parse_imports(code.as_str())
                .into_iter()
                .map(|i| resolve_import(&path, &i))
                .collect::<Vec<PathBuf>>();
            files.insert(path, code);
            for imp in imports {
                if !files.contains_key(&imp) && !import_queue.contains(&imp) {
                    import_queue.push(imp)
                }
            }
            import_files(files, import_queue);
        }
    }
}

pub fn resolve_import(current_file: &PathBuf, import: &str) -> PathBuf {
    let import_path = PathBuf::from(import).with_extension("arena");

    // Check at current_file location:
    let current_folder = current_file.parent().unwrap_or_else(|| {
        panic!(
            "Could not get folder containing {}",
            current_file.to_str().unwrap()
        )
    });
    fs::canonicalize(current_folder.join(&import_path)).unwrap_or_else(|_| {
        // Check at ~/.arena/lib:
        let home = std::env::var("HOME").expect("Could not get HOME directory");
        let lib_folder = PathBuf::from(home).join(".arena/lib").join(import_path);
        fs::canonicalize(&lib_folder)
            .unwrap_or_else(|_| panic!("Could not resolve import path of {:?}", lib_folder))
    })
}

pub fn resolve_all_imports<'input>(
    asts: HashMap<&PathBuf, Module<'input>>,
    main_module_path: &PathBuf,
) -> (HashMap<Rc<String>, Module<'input>>, Rc<String>) {
    // returns (modules, main_module_id)

    let mut module_prefixes = HashMap::new();
    (
        asts.into_iter()
            .map(|(path, module)| {
                (
                    get_module_prefix(path, &mut module_prefixes),
                    Module {
                        imports: module
                            .imports
                            .into_iter()
                            .map(|(name, imp_path)| {
                                (
                                    name,
                                    get_module_prefix(
                                        &resolve_import(path, &imp_path),
                                        &mut module_prefixes,
                                    ),
                                )
                            })
                            .collect(),
                        types: module.types,
                        functions: module.functions,
                    },
                )
            })
            .collect(),
        Rc::clone(&module_prefixes[main_module_path]),
    )
}

fn get_module_prefix(
    module: &PathBuf,
    module_prefixes: &mut HashMap<PathBuf, Rc<String>>,
) -> Rc<String> {
    match module_prefixes.get(module) {
        None => {
            let mut hasher = DefaultHasher::new();
            module.hash(&mut hasher);
            let hash = format!("${:x}$", hasher.finish());
            module_prefixes.insert(module.to_path_buf(), Rc::new(hash));
            Rc::clone(module_prefixes.get(module).unwrap())
        }
        Some(name) => Rc::clone(name),
    }
}
