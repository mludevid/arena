# Hello World

When learning a new programming language it is mandatory to write a `Hello World`
program. For `Arena` it looks as follows:

Filename: hello_world.arena
```
fn main() = print("Hello World!\n")
```

To compile the program you need to call the `Arena` compiler and pass it the
correct filename. Afterwards you can execute the binary. The default binary name
is `out`.

```bash
$ arena hello_word.arena
$ ./out
```
