# Module

Each file with the extension `.arena` is interpreted by the `Arena` compiler as
a module.

Each module consists of imports at the top of the file and arbitrary many function
definitions. The root module has to contain a `main` function which is the
entry-point of the compiled binary.
