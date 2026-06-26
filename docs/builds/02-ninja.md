# Ninja output

When `pb` processes values created by `pb.rule` and `pb.build`, it typically emits a Ninja build file. Understanding how Lead concepts map to Ninja helps reason about performance and incremental builds.

## Mapping summary

- Lead rule created with `pb.rule` -> Ninja rule
- Lead build created with `pb.build` -> Ninja build line
- Rule fields such as `command` and `description` become Ninja rule attributes
- Inputs and outputs from the build become the corresponding Ninja edge

Example: Lead snippet and corresponding Ninja fragment

```lead
|{pb, ...}|
let
  cc_rule = pb.rule {
    name = "cc";
    cmd = "gcc -c {in} -o {out}";
    description = "compile {in}";
  };

  obj_build = pb.build {
    rule = cc_rule;
    in = "src/main.c";
    out = "build/main.o";
  };
in
  obj_build
```

Illustrative Ninja output:

```ninja
rule cc
  command = gcc -c $in -o $out
  description = compile $in

build build/main.o: cc src/main.c
```

Notes:

- `pb` may generate additional Ninja features such as depfiles, rspfiles, or pools depending on the rule fields and flags.
- Inspect the generated `build.ninja` in your project’s build directory to see the exact translation.
- The examples directory contains end-to-end samples that show how these values are represented in practice.
