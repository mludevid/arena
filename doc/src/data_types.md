# Data Types

Arena currently has 4 primitive data types:

- `void` is the empty type
- `i32` is a signed 32 bit integer
- `bool` is a boolean and can therefore either be `true` of `false`
- `str` is a literal String and is defined between two double quotes

The user can also define own types. These work similar user defined types in Haskell where the keyword `data` is used. Each type can have several cases which can contain fields to save data. The instances of these types are always allocated on the heap. The definition of a linked list of integers could look as follows:

```
type IntList {
    Nil,
    Cons(i32, IntList),
}
```

To construct a new instance you need to specify a case of the type and initialize all its fields:

```
let x = IntList.Cons(1, IntList.Cons(2, IntList.Cons(3, IntList.Nil)));
...
```

To access the fields of a user defined type you have to deconstruct the type with a `match`. Look into the chapter about [Control Flows](control_flow.md) to learn more about them.

As mentioned before all used defined types are always allocated on the heap. The garbage collection strategy used to maintain the heap can affect the memory layout and execution time but will never alter the execution result. You can specify one of the three currently available garbage collection strategies with a compile time flag:

```bash
$ arena example.arena --spill  # will not free any memory until the program terminates
$ arena example.arena --arc    # automatic reference counting
$ arena example.arena          # arc is the Default
$ arena example.arena --tgc    # tracing garbage collection (mark and copy collector)
```
