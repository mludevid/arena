use std::process::Command;

fn main() {
    lalrpop::process_root().unwrap();
    let gcc_output = Command::new("gcc")
        .arg("-Wextra")
        .arg("-c")
        .arg("runtime/libarena.c")
        .arg("-o")
        .arg("libarena.o")
        .output()
        .expect("Failed to execute gcc");
    if !gcc_output.status.success() {
        panic!(
            "gcc failed with error code {:?}. Output:\n{}",
            gcc_output.status.code(),
            String::from_utf8_lossy(&gcc_output.stderr)
        )
    }
    let gcc_output = Command::new("gcc")
        .arg("-Wextra")
        .arg("-DPROFILING_STACK")
        .arg("-c")
        .arg("runtime/libarena.c")
        .arg("-o")
        .arg("libarena_prof_stack.o")
        .output()
        .expect("Failed to execute gcc");
    if !gcc_output.status.success() {
        panic!(
            "gcc failed with error code {:?}. Output:\n{}",
            gcc_output.status.code(),
            String::from_utf8_lossy(&gcc_output.stderr)
        )
    }
    let gcc_output = Command::new("gcc")
        .arg("-Wextra")
        .arg("-c")
        .arg("runtime/profiling.c")
        .arg("-o")
        .arg("profiling.o")
        .output()
        .expect("Failed to execute gcc");
    if !gcc_output.status.success() {
        panic!(
            "gcc failed with error code {:?}. Output:\n{}",
            gcc_output.status.code(),
            String::from_utf8_lossy(&gcc_output.stderr)
        )
    }
    let ar_output = Command::new("ar")
        .arg("rc")
        .arg("libarena.a")
        .arg("libarena.o")
        .arg("profiling.o")
        .output()
        .expect("Failed to execute ar");
    if !ar_output.status.success() {
        panic!(
            "ar failed with error code {:?}. Output:\n{}",
            ar_output.status.code(),
            String::from_utf8_lossy(&ar_output.stderr)
        )
    }
    let ar_output = Command::new("ar")
        .arg("rc")
        .arg("libarena_prof_stack.a")
        .arg("libarena_prof_stack.o")
        .arg("profiling.o")
        .output()
        .expect("Failed to execute ar");
    if !ar_output.status.success() {
        panic!(
            "ar failed with error code {:?}. Output:\n{}",
            ar_output.status.code(),
            String::from_utf8_lossy(&ar_output.stderr)
        )
    }
    let rm_output = Command::new("rm")
        .arg("libarena.o")
        .output()
        .expect("Failed to execute rm");
    if !rm_output.status.success() {
        panic!(
            "rm failed with error code {:?}. Output:\n{}",
            rm_output.status.code(),
            String::from_utf8_lossy(&rm_output.stderr)
        )
    }
    let rm_output = Command::new("rm")
        .arg("libarena_prof_stack.o")
        .output()
        .expect("Failed to execute rm");
    if !rm_output.status.success() {
        panic!(
            "rm failed with error code {:?}. Output:\n{}",
            rm_output.status.code(),
            String::from_utf8_lossy(&rm_output.stderr)
        )
    }
    let rm_output = Command::new("rm")
        .arg("profiling.o")
        .output()
        .expect("Failed to execute rm");
    if !rm_output.status.success() {
        panic!(
            "rm failed with error code {:?}. Output:\n{}",
            rm_output.status.code(),
            String::from_utf8_lossy(&rm_output.stderr)
        )
    }
}
