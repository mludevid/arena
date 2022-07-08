# Module

Each file with the extension `.arena` is interpreted by the `Arena` compiler as a module.

Each module consists of imports at the top of the file, afterward type definitions and finally arbitrary many function definitions. The compiler needs a `main` function in the provided module as the entry-point of the compiled binary, this is not necessary for imported modules.
