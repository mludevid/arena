# Imports

Every module can have arbitrary many imports statements at the top of the file.

There are three options to import another module. If the file is located in the
same directory as the current one and the file name is an allowed identifier in
arena (contains letters, numbers, and underscores and does not start with a
number) then you can import the module as follows:

```
import other_module

// or if you want to use the module with the name 'lib':

import other_mode as lib
```

If you need to define a relative path or the name is not an allowed indentifier
you need to use quotes around the import path and you need to give the imported
module a name to be used by in the current module:

```
import "subfolder/other_module" as lib
```

All functions and types defined in the other module can now be used in the
current module by prefixing the function or type name with the identifier
specified during importation separatied by two colon. For example, to call a
function `foo` defined in the `other_module` which was given the identifier
`lib` during its importation would work like this:

```
import other_module as lib

fn main() = lib::foo()
```

If the imported file is not found relative to the current path, the compiler will search for it in two other locations: the user wide library folder: `~/.arena/lib/` and the system wide library collection in the lib subfolder of the installation directory of the arena compiler.
