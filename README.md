# Arena

This is the code base for the Arena compiler.

Arena is a natively compiled functional programming language, that is being
developed with the intend to benchmark and compare automatic garbage collection
algorithms.

The documentation of the language can be found
[here](https://mludevid.github.io/arena/).

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

You also need compilation tools from `build-essential` and the `Zlib` library.
If they are missing on your Ubuntu machine, you can install both packages with
the commands:

```bash
$ sudo apt install build-essential
$ sudo apt install zlib1g-dev
```

You should be ready to build the project:

```bash
$ ./build.sh
```

This will call `cargo build` to build the project which generates a debug
version of the compiler at `./target/debug/arena`. The compiler is then
automatically copied to the root directory of the project. This is needed,
because the compiler is going to search for the arena libraries directory in the
`lib` folder and the `libarena.a` comiler library in the same folder as the
compiler itself. The testing framework (`test.py`) also requires the compiler
to be in the root directory.

If the building process does not automatically find the LLVM utilities you may
have to indicate the location of the installation. If you followed this guide it
should be located in `/usr/bin`. Therefore you can build the project with:

```bash
$ LLVM_SYS_100_PREFIX=/usr/bin cargo b
```

For a release version of the compiler use:

```bash
$ ./build.sh -r
```

The compilation will take longer but the compiler will work faster.

If you want to be able to use the compiler from anywhere on your sistem you can
use the installation script which automatically creates a symbolic link in the
`/usr/bin` folder to the compiler so that your shell can find it from anywhere.


```bash
$ ./install.sh
```

To delete this symbolk link you can use the uninstall script:

```bash
$ ./uninstall.sh
```
