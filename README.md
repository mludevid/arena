# Arena

This is the code base for the Arena compiler.

Arena is a natively compiled functional programming language, that is being
developed with the intend to benchmark and compare automatic garbage collection
algorithms.

## Installation
To build the project you will need a rust compiler. The easiest way to install
it on Linux is to use the `rustup` tool:

```bash
$ curl https://sh.rustup.rs -sSf | sh
```

You also need a working installation of the LLVM toolchain. It is important to
make sure that version 10 is installed. To install it on Ubuntu you can simply
run:

```bash
$ sudo apt install llvm
```

Other libraries are required for the linking of the project. On an Ubuntu
machine with `build-essential` installed you should only be missing `Zlib`.
Install it with:

```bash
$ sudo apt install zlib1g-dev
```

You should be ready to build the project:

```bash
$ cargo build
```

This will generate a debug version of the compiler at `./target/debug/arena`.

You can use it with `./target/debug/arena` or copy it to the root directory to
be able to execute it with `./arena`. The testing framework (`test.py`) requires
the compiler to be in the root directory. If you rebuild the compiler often we
also recommend creating a symbolic link from `./arena` to
`./target/debug/arena`.

If the building process does not automatically find the LLVM utilities you may
have to indicate the location of the installation. If you followed this guide it
should be located in `/usr/bin`. Therefore you can build the project with:

```bash
$ LLVM_SYS_100_PREFIX=/usr/bin cargo b
```
