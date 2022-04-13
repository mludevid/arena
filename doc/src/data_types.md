# Data Types

Arena has 4 primitive data types:

- `void` is the empty type
- `i32` is a signed 32 bit integer
- `bool` is a boolean and can therefore either be `true` of `false`
- `String` is a literal String and is defined between two double quotes

The user can also define own types. These work similar to `data` in Haskell for
example. Each type can have several type cases which can contain fields to save
data. The instances of these types are always allocated on the heap. The
definition of a linked list of integers could look as follows:

```
type IntList {
    Nil,
    Cons(i32, IntList),
}
```

To construct simply initialize all fields of a type case:

```
let x = IntList.Cons(1, IntList.Cons(2, IntList.Cons(3, IntList.Nil)));
...
```

To access the fields of a user defined type you have to deconstruct the type
with a `match`. Look into the chapter about [Control Flows](control_flow.md) to
learn more about them.
