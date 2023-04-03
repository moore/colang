# Typed VM

The typed VM has an instruction set that assumes that is built for static dispatch and keeping vars on the stack using a frame pointer that can be usd with constant offsets.

```
[ ... ]
[var n]
  ...
[var 1]
[arg n]
  ...
[arg 1] <- frame pointer
[ ... ]
[ ... ]
[ ... ]
```

This is supported by `Load` and `Store` instructions that work with the frame pointer. Ex the instruction sequence
```
(Usize:0)
(Load   )
 ```
 would push a `0` on to the stack and then execute `Load` which would consume the `0` and copy `[arg 1]` to the top of the stack. This is because arg 1 is at the `0` offset from the `frame pointer`. All variables and arguments will have a constant offset from the frame pointer making reading and writhing them simple.

 ## Structs
 Any sequential group of stack values can be considered a struct. That is to say structs are just a way of interpreting the stack.
 
  To make this representation easy to work with instructions that manipulate the stack come in the default version which works with a single stack value and `N` variants. For example `(pop)` would remove the top value in the stack.
 `(popN)` would treat the top value in the stack as a count of items to remove and would remove the count as well as n more entries for the stack where n is the value of the count.

 Given that structs are just sequences of stack values `popN` can be used to pop off a struct value in a single operation.