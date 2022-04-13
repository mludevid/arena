# Control Flow

Currently there is only one control flow construct: the `if` expression:

## `if` expression

As usual for all functional programming languages all control flow constructs
are expressions instead of statements and therefore always return a value. This
is not different for `Arena`. We can use this construct and the knowledge
acquired in previous chapters to calculate the 40th fibonacci number:

```
fn main() = fib(40)

fn fib(n: i32) -> i32 =
    if n < 1 then
        n
    else
        fib(n-1) + fib(n-2)
```

If either the `then` or the `else` block include an expression with a semicolon
you have to surround the block with curly brackets to resolve possible
ambiguities of the location of the end of the block.

Alternatively you can use the fact, that in `Arena`paranthesis can include any
expression. This allows you to surround the block that includes a semicolon with
paranthesis instead of surrounding both with curly brackets.

```
fn print_n_times(str: String, n: i32) =
    if n <= 1 then {
        print(str)
    } else {
        print(str);
        print_n_times(str, n-1)
    }

// Or:

    if n <= 1 then
        print(str)
    else (
        print(str);
        print_n_times(str, n-1)
    )
```

## `match` expression

The `match` keyword also starts an expression and is mainly used to deconstruct
user defined types. Each `match` expression takes an object and tries to match
it against a set of patterns. The body of the first pattern tht succeeds will
be executed populating the variables defined in the pattern with the
corresponding values. This example shows how a spaced print could be implemented for
a linked list of words.

```
type List {
    Nil,
    Cons(String, List),
}

fn printList(l: List) =
    match l {
        List.Nil => print("\n"),
        List.Cons(w, List.Nil) => print(w); print("\n"),
        List.Cons(w, tail) => print(w); print(" "); printList(tail),
    }
```
