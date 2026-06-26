# Rules and Builds

Rules and builds are special value types in Lead. They are not ordinary literals; they are created using the builtin constructors `pb.rule` and `pb.build`.

## Rules

A rule describes how to transform inputs into outputs. In Lead, a rule is constructed with `pb.rule` and may contain fields such as the command template, description, and other rule metadata.

## Builds

A build is an instance of a rule applied to concrete inputs and outputs. In Lead, a build is constructed with `pb.build` and references a rule plus the relevant inputs, outputs, and parameters.

Minimal example:

```lead
|{pb, cwd, ...}|
let
  cc_rule = pb.rule {
    name = "cc";
    cmd = "gcc -c {in} -o {out} -I{inc}";
    description = "compile {in}";
  };

  my_build = pb.build {
    rule = cc_rule;
    in = cwd / "src" / "main.c";
    out = cwd / "build" / "main.o";
    inc = cwd / "include";
  };
in
  my_build
```

Notes:

- `pb.rule` produces a rule object.
- `pb.build` produces a build object.
- The examples directory in the repository contains sample `.pbb` files that define real rules and builds; consult them for fuller patterns.
- Rules and builds remain values until `pb` emits the build graph (Ninja) and/or executes it.
