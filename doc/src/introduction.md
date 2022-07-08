# Introduction

`Arena` is a typed, functional and natively compiled programming language. It was created to compare garbage collection algorithms. To be fair battleground for the competing algorithms it does not have any optimizations regarding memory management. Even though the language is kept as minimalistic as possible to simplify the development of the compiler it does implement all core features of a typed, functional programming language: it has immutable variables, user defined types, pattern matching, functions, type checking and an import system.

Currently `Arena` features three garbage collection algorithms that can be selected at compile time:
- `Spill` as the name suggest allocates all objects on the heap with a call to the `malloc` function provided in `libc` and then never frees this storage until the program terminates.
- `ARC` stands for automatic reference counting and is the default garbage collection algorithm. Every allocated object on the heap maintains a counter of the existing pointer to it. When this counter reaches 0 the object is no longer accessible and can therefore be freed.
- `TGC` is the tracing garbage collection algorithm, currenlty a basic mark and copy collector. The available heap space is divided into two parts: an active and a copy heap. When the active heap is full a garbage collection pass is triggered which traverses the stack and copies all reachable objects to the copy heap. Afterwards the roles are swiched and the copy heap becomes the active one until the next collection.
