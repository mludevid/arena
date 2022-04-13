# Operators

`Arena` has the following primitive operators:

- `+` Adds two integers: `2 + 3`
- `-` Subtracts two integers or negates one: `2 - 3` or `-2`
- `*` Multiplies two integers: `2 * 3`
- `/` Divides two integers and returns the integer result: `2 / 3` returns `0`
- `%` Divides two integers and returns the remainder: `2 % 3` returns `2`
- `!` Negates a boolean: `!true` returns `false` and `!false` returns `true`
- `||` Returns the logical or of two booleans: `false || true` *
- `&&` Returns the logical and of two booleans: `true && false` *
- `==` Checks the two operands for equality: `2 == 3`
- `!=` Checks the two operands for inequality: `2 != 3`
- `<` Checks if the first integer is smaller than the second one: `2 < 3`
- `<=` Checks if the first integer is smaller than or equal to the second one: `2 <= 3`
- `>` Checks if the first integer is larger than the second one: `2 > 3`
- `>=` Checks if the first integer is larger than or equal to the second one: `2 >= 3`

\* Both logical operators are lazy and only compute the second operant if
necessary
