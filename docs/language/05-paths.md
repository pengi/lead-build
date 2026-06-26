# Paths

Paths are objects in the language that represent a location in the filesystem. A path value may refer to either a file or a directory.

Paths are typically obtained from builtin functions or other language constructs. The builtin `cwd` represents the directory of the current `.pbb` file and is commonly used as the starting point for path traversal. Every example below includes a file header that brings `cwd` into scope.

## Traversal

Use the `/` operator to move down into a directory or file name:

```lead
|{cwd, ...}|
let
  src = cwd / "src";
  main = src / "main.c";
in
  main
```

In this example, `src` is the path one level below `cwd`, and `main` is a path below `src`.

The right-hand side of `/` must be a string representing a child name.

## Upward traversal

You can also move upward using the special segment `".."`:

```lead
|{cwd, ...}|
let
  src = cwd / "src";
  back = src / "..";
in
  back
```

This returns the parent directory of `src`, but not above the original `cwd` origin.

## Bound root

Upward traversal is limited: a path cannot move higher than the original directory where it was defined. The path value is anchored at its original root, so repeated `"/.."` operations will stop at that root and will not escape above it.

This keeps path traversal safe and predictable while still allowing relative movement within the defined path object.

## Locking a path

The builtin `pb` contains a function called `lock` that creates a new path value bound to the same file or directory, but with a fresh root boundary.

```lead
|{cwd, pb, ...}|
let
  locked = pb.lock (cwd / "src");
  parent = locked / "..";
in
  parent
```

In this example, `locked` refers to the same directory as `cwd / "src"`, but its upward traversal is restricted to that path. The example is intended to show that attempting `locked / ".."` does not escape above the locked root and will fail.
