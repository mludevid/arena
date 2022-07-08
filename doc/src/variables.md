# Variables

Like most programming languages `Arena` also features variables. But as all
functional languages this name is not really appropiate as they are immutable.

They work as one may expect: first the variables value is defined and afterwards
this value can be accessed by the variables name:

```
fn main() =
    let x = 10;
    if x == 10 then
        print("x has the value 10\n")
    else
        print("something went very wrong\n")
```

This is also the first time that we see the semicolon `;`. It is compulsory
after variable definitions as the definition itself does not return any value
and the expression is therefore not complete yet. But it can also be used to
concatenate two expressions. This does only make sense if the first expression
has side effects. Pure functional programming languages in a mathematical sense should not have any side effects but
all languages end up having some mainly through interactions with `IO`:

```
fn main() =
    print("Hello ");
    print("World!");
    print("\n")
```

It's important to keep in mind that parenthesis in `Arena` also function as a
namespace. Therefore all variables defined inside a parenthesis block can not be
accessed after the namespace is closed and exited:

```
fn main() =
    // accessing y outside of the then block would result in an compilation error
    let x = if true then (let y = 1; 2) else 3;
    x
```

The main function would exit with the value 2.
