# Lead Build Language

This guide introduces the Lead Build language from first principles. It walks through the core syntax, expressions, functions, pattern matching, and iteration, so later chapters build naturally on the earlier ones.

## Evaluation and testing

To be able to test how complex evaluations work, it is possible to evaluate an expression without generating a build file. This is useful when testing language concepts and debugging.

To evaluate a file, for example:

```lead
|{...}|
let
  a = 13;
in
{
  variable = a;
}
```

run `pb` with the `-E` argument, which triggers evaluation:

```sh
pb -E -i myfile.pbb
```

It should output something similar to:

```lead
{
  variable = 13;
}
```

You may notice the leading `|{...}|` line in the example above. It is part of the file header and will be explained later, once we have introduced builtins and the `include` mechanism. For now, treat it as a small wrapper around the expression being evaluated.

We’ll break down how the language is actually structured in the following chapters.