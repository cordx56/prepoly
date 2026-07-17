---
title: "Hello, world!"
description: "Your first Brass program: printing, running, and checking."
---

Let's write your first Brass program!

Write the following program into a `hello.cz` file.

```brass
println("Hello, world!")
```

Then, execute the program:

```bash
brass hello.cz
```

The output is as follows:

```
Hello, world!
```

A Brass source file is a script: top-level statements run from top to bottom.
You can also define a `main` function, which is called after the top-level
statements have run:

```brass
fun main() {
    println("Hello, world!")
}
```

The execution result is the same as the previous one.

## Checking without running

Every function is fully type-checked before it runs. To type-check a program
without executing it, use `brass check`:

```bash
brass check hello.cz
```

It prints nothing when the program is well-typed, and exits 0; otherwise it
prints the type errors and exits non-zero.
Running `brass` with no arguments starts an interactive REPL.

### A run only checks what it runs

`brass hello.cz` checks **lazily**: the run's verdict covers the code the run
actually needs. A type error in a function the run never calls does not stop
the run:

```brass norun
fun broken() -> int32 {
    return "oops"    // a type error -- but nothing calls broken()
}

println("Hello, world!")
```

`brass hello.cz` prints `Hello, world!` and exits 0. `brass check hello.cz`
reports the error in `broken`.

What this means in practice:

- **A green run is not a full type check.** Use `brass check` where you want
  the whole-program verdict — in CI, before a commit, after a refactor.
- Errors in code the run does need still stop it: an error in the top-level
  statements or in `main` aborts before anything executes, and an error in a
  function first reached mid-run stops the run right there — output already
  produced by then stands, and the run exits non-zero. The unit is the whole
  function: an error anywhere in a function the run calls counts, even in a
  branch execution would never take.
- The unused code keeps checking in the background while your program runs;
  what it finds is remembered to speed up later runs, but it never changes
  the current run's outcome.
- `brass --eager hello.cz` runs with the check-everything-first behavior
  (identical to `brass check` followed by the run). The REPL and the
  interpreter back end always check eagerly.

See [Execution model](/references/execution/) for the full rules.

## Running as a script

`#` starts a line comment, so a source file may begin with a **shebang** line.
That lets you run a Brass file directly, like a shell script:

```brass
#!/usr/bin/env brass
println("Hello from a script!")
```

Mark the file executable once, then run it by name:

```bash
chmod +x hello.cz
./hello.cz
```

The shebang line is ordinary comment syntax, so the same file still works with
`brass hello.cz`, `brass check hello.cz`, and on systems that do not use
shebangs at all.

## GCD: Greatest Common Divisor

Next, let's write a practical example.

We can write a `gcd` function, which calculates the greatest common divisor,
as follows:

```brass
fun gcd(a, b) {
    if b == 0 {
        return a
    } else {
        return gcd(b, a % b)
    }
}

println(gcd(48, 36))
```

This outputs `12`, which is correct!

Note that we didn't write a single type annotation: parameter and return types
are inferred. The program is still statically typed — passing a string to
`gcd` would be rejected before execution.

## Variables and arrays

We can use `const` to declare an immutable variable and `let` to declare a
mutable variable.

```brass
const pi = 3.14159   // reassigning is a compile error
let count = 0
count += 1
```

The following program calculates the gcd of all elements in an array:

```brass
fun gcd(a, b) {
    if b == 0 {
        return a
    } else {
        return gcd(b, a % b)
    }
}

const elems = [16, 36, 72, 192]
let result = elems[0]
for elem in elems.slice(1, elems.len()) {
    result = gcd(result, elem)
}
println("GCD is {result}")
```

This program outputs `GCD is 4`.

The `{result}` inside the string is **string interpolation**: `{expr}`
evaluates the expression and inserts its text into the string.

Now you have seen a complete little program. The following chapters introduce
each language feature the same way — by example. For exhaustive rules, see the
[references](/references/syntax/).
