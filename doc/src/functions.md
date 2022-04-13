# Functions

A function in `Arena` has to have a defined set of input parameters with their
types specified, a return type and the body of the function.
For example, a function `between` that checks if a number `a` lays between two
numbers `b` and `c` would be defined like this:

```
fn between(a: i32, b: i32, c: i32) -> bool = a > b && a < c
```

Function calls are similar to other mainstream languages by passing the
arguments between round brackets. To check if `1` is smaller than `3` and larger
than `-1` we type:

```
between(1,-1,3)
```

A function can return `void` which will drop the last computed value and not
return anything. `void` is the default return type of a function and it is
therefore not mandatory to specify.

```
fn println(str: String) -> void = print(str); print("\n")

// Or:

fn println(str: String) = print(str); print("\n")
```
