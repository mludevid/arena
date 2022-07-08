# Functions

A function in `Arena` has to have a defined set of typed input parameters, a return type and the body of the function. For example, a function `between` that checks if a number `a` lays between two numbers `b` and `c` would be defined like this:

```
fn between(a: i32, b: i32, c: i32) -> bool = a > b && a < c
```

The instructions after the `=` are the body of the function. Given a specific instantiation of the function parameters the body outputs a value of the return type of the function. If the return type is `void` the function does not return anything.

Function calls are performed similar to other mainstream languages by passing the arguments between round brackets. To check if `1` is smaller than `3` and larger than `-1` you can use:

```
between(1, -1, 3)
```

A function with return type `void` will drop the last computed value and not return anything. `void` is the default return type of a function and can be omitted in the function definition:

```
fn println(s: str) -> void = print(s); print("\n")

// Or:

fn println(s: str) = print(s); print("\n")
```
