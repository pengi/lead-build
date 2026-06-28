# Functions and Pattern Matching

Functions make it possible to reuse behavior and patterns.

## Functions

Functions are themselves a compound type, which can be invoked to transform an input argument into an output.

They use the syntax: `|argument| output`

Here, `argument` is a variable pattern, of the same form as in `let ... in ...` statements described in the previous chapter.

They are normally assigned to a variable or an element in an object, to be invoked later. But since they are values themselves, they can also be passed as arguments or stored in lists.

Invocations are done by referencing the function followed by the argument. For example: `<func> <arg>`.

For example:

```lead
let
  add_three = |x| x + 3;
in
  {
    a = add_three 1;
    b = add_three 2;
    c = add_three 3;
  }
```

This evaluates to:

```lead
{
  a = 4;
  b = 5;
  c = 6;
}
```

Functions do not have to be stored in variables. It is valid to call them directly. For example, `(|a| a + 10) 3` evaluates to `13`.

## Scopes and bound variables

When declaring functions, values from the scope where they are declared may be referenced from the function.

```lead
let
  add_val = 3;
  add_three = |x| x + add_val;
in
  {
    a = add_three 1;
    b = add_three 2;
    c = add_three 3;
  }
```

This is still valid. When `add_three` is created, it has `add_val = 3` in its scope, which is bound to the function.

Since the scope follows the function and cannot be updated later, it is still pure: the behavior of the function will be the same wherever it is called and will depend only on its input.

## Multiple arguments

A function by design only takes one argument and returns a single value. But often it is necessary to take multiple arguments or return multiple values.

Returning multiple values is achieved by using one of the compound types, typically an object, but lists and tuples are also possible.

To make a function take multiple arguments, it is also possible to take a compound type, which can be helped by variable patterns and pattern matching, as described later.

It is also possible to leverage the concept of bound scopes, as described above, by making a function return another function.

For example:

```lead
let
  add = |a| |b| (a + b);
in
  (add 3 5)
```

In the example above, `add 3 5` is resolved as `(add 3) 5`. The expression `add 3` is evaluated using the outer function, returning `|b| (a + b)`, where `a` is bound to `3` from the input parameter and can therefore be written as `|b| (3 + b)`.

When replacing `add 3` with the value above, `(|b| (3 + b)) 5` evaluates to `3 + 5`, or `8`.

As syntactic sugar, it is possible to skip the middle `|`, writing multiple arguments as `|a b| (a + b)`.

This makes it possible to create new functions where earlier arguments are already set. For example:

```lead
let
  add = |a b| a + b;
  add_three = add 3;
in
  (add_three 5)
```

## Pattern matching

The other method of sending multiple arguments to the same function is by using compound types. To make it more convenient to select and unpack the compound type into individual variables, it is possible to pattern match on the types.

The same mechanism is used in the `let ... in ...` construct when assigning variables, as described in earlier chapters.

A pattern can be one of:

- variable - a single name, representing the entire argument
- tuple pattern - written as a tuple `(a, b, c)`, where each field is itself a pattern, expecting a tuple of the correct size
- object pattern - written as a pattern surrounded by braces `{...}`, containing a set of match statements separated by commas
- null pattern - a single `_`, matching anything, but ignoring the value and not storing it

An object pattern contains a set of match statements, which can either be:

- a single name, matching the name in the object to a variable of the same name
- a string `name = submatcher`, matching the field `name` in the input object using the `submatcher`. This means `{ field = varname; } = obj;`, unpacking `obj` to place the field `field` into the variable `varname`

Optional default values can be added with `name ? default_expr` or `name = submatcher ? default_expr`. If the field is missing, the default expression is used.

If a field is missing and no default is defined for that field, matching fails with an error.

An object pattern normally expects all fields to be extracted; otherwise it will return an error. If only a few fields are expected to be unpacked but more fields should be allowed, the object matcher can end with a `...` statement, allowing extra fields.

Any matcher, except single-name object matchers, can be combined with `matcher @ variable`, causing the entire field to be stored in the variable while also being matched by the matcher. This is useful when extracting some fields of an object while still having access to the object. For example: `|{var, ...} @ obj| ...`.

For example:

```lead
let
  myfunc = |{name, ...} @ obj| { name = name; type = "obj"; inner = obj; };
in
  myfunc {
    name = "test";
    local_var = "something";
  }
```

This evaluates to:

```lead
{
  inner = {
    local_var = "something";
    name = "test";
  };
  name = "test";
  type = "obj";
}
```

This shows that the original argument is copied in its entirety to `inner` in the result, while `name` is unpacked.

This can also be applied to tuples:

```lead
let
  add = |(a, b)| a + b;
in
  add (3, 4)
```

This evaluates to `7`.

Default values are especially useful for optional object fields in matcher patterns:

```lead
let
  add_with_default = |{a, b ? 12}| a + b;
in
  add_with_default { a = 3; }
```

This evaluates to `15`.

## File header and builtins

At this point, all the pieces are there to revisit the file header.

Looking at the first example:

```lead
|{...}|
let
  a = 13;
in
{
  variable = a;
}
```

it is quite clear that the file actually contains a function taking an object as its argument and matching any extra unspecified fields.

That is because the file itself contains a single pure statement. Any builtins are passed as arguments to the function. Builtins are functions or arguments built into the `pb` tool itself and exposed to the implementation.

For example, to have access to the `include` function described later, it is possible to pattern match on the field `include` in the header:

```lead
|{include, ...}|
...
```

This section only gives a brief preview of the file header and builtins. The exact details and the role of the `include` function will be explained in later chapters.