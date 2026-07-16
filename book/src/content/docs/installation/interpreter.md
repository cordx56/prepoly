---
title: "Installing Brass"
description: "Build and install the Brass command-line driver."
---

Here we describe how to build and install the Brass command-line driver.

## Clone the repository

First, clone the Brass source code:

```bash
git clone https://github.com/cordx56/prepoly.git
```

## Install the Rust compiler

Brass is written in the Rust language.
So first you have to install the Rust compiler.

You can learn how to install Rust here: [https://rust-lang.org/tools/install/](https://rust-lang.org/tools/install/)

## Build Brass with the native runtime

After installing Rust, you can build the default native driver:

```bash
./x cargo build --release
```

The script `x` builds the `bootstrap` crate and executes commands via the `bootstrap` binary.
The `bootstrap` binary downloads LLVM, which is required to use JIT compilation, and sets its path for the Brass build.

The resulting binary `brass` is made under the `target/release` directory.

If you want the interpreter-only driver without LLVM, disable default features:

```bash
cargo build -p brass_driver --no-default-features
```

## Install Brass

Run the following command to install the default native driver:

```bash
./x cargo install --path crates/brass_driver
```

Note that you have to add the path where the `brass` binary is placed to `$PATH`.

## Usage

```bash
brass program.cz     # type-check and run a program
brass check program.cz   # type-check only; silent on success
brass repl program.cz    # run a program with the interpreter (no JIT)
brass                # start an interactive REPL
```

Any diagnostic (parse error, type error) is printed to stderr and the process
exits with a non-zero status; nothing is executed unless the whole program
checks.
