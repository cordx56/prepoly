//! End-to-end tests for the REPL runtime: the `prepoly_repl` interpreter reached
//! through `prepoly repl`.
//!
//! The interpreter is held to the same observable behavior as the LLVM JIT for the
//! typed sequential subset, so the main test runs each case through *both*
//! `prepoly run` (JIT) and `prepoly repl` (interpreter) and asserts their stdout is
//! identical -- a self-validating parity check that needs no recorded `.out`
//! files. A second test pins exact output for a file run, and a third drives an
//! interactive session over stdin (definitions, statements, and an echoed bare
//! expression).

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn repo_root() -> String {
    // The driver manifest lives at crates/prepoly_driver; the repo root is two up.
    format!("{}/../..", env!("CARGO_MANIFEST_DIR"))
}

/// Run a file through a back end and return (success, stdout, stderr). `mode`
/// `"run"` uses the default runtime via a bare file argument (no subcommand);
/// any other mode (e.g. `"repl"`) is passed as the subcommand.
fn run_mode(mode: &str, path: &str) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_prepoly"));
    if mode != "run" {
        cmd.arg(mode);
    }
    let out = cmd.arg(path).output().expect("spawn prepoly");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Examples within the interpreter's supported subset (everything but the
/// concurrency and file-I/O examples, which exercise runtime features the REPL
/// does not implement). The JIT and the REPL must produce identical output.
const PARITY_CASES: &[&str] = &[
    "examples/01_records.pp",
    "examples/02_sum_types.pp",
    "examples/03_interfaces.pp",
    "examples/04_sum_interface.pp",
    "examples/05_nullable_and_result.pp",
    "examples/06_structural_subtyping.pp",
    "examples/07_closures.pp",
    "examples/08_pattern_matching.pp",
    "examples/09_collections.pp",
    "examples/10_strings_and_conversions.pp",
    "examples/11_control_flow.pp",
    "examples/14_type_safety.pp",
    "examples/15_numeric_conversions.pp",
    "examples/16_method_inference.pp",
    "examples/17_higher_order.pp",
    "examples/18_methods.pp",
    "examples/modules/main.pp",
];

#[test]
fn repl_matches_jit_on_supported_examples() {
    let root = repo_root();
    for ex in PARITY_CASES {
        let path = format!("{root}/{ex}");
        let (jit_ok, jit_out, jit_err) = run_mode("run", &path);
        let (repl_ok, repl_out, repl_err) = run_mode("repl", &path);
        assert!(jit_ok, "JIT run failed for {ex}\nstderr:\n{jit_err}");
        assert!(repl_ok, "REPL run failed for {ex}\nstderr:\n{repl_err}");
        assert_eq!(
            jit_out, repl_out,
            "REPL and JIT stdout differ for {ex}\nJIT:\n{jit_out}\nREPL:\n{repl_out}",
        );
    }
}

#[test]
fn repl_runs_file_with_expected_output() {
    // A self-contained program (recursion, arrays, a `for` loop, string
    // interpolation, float formatting) run through the REPL interpreter, pinned to
    // its exact stdout so the interpreter's arithmetic and rendering are fixed, not
    // just matched against the JIT.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("repl_file");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("prog.pp");
    fs::write(
        &path,
        "fun fib(n: int32) -> int32 {\n\
         \x20   if n < 2 { return n }\n\
         \x20   return fib(n - 1) + fib(n - 2)\n\
         }\n\
         fun main() {\n\
         \x20   println(\"fib(10) = {fib(10)}\")\n\
         \x20   let xs = [4, 8, 15, 16, 23, 42]\n\
         \x20   let total = 0\n\
         \x20   for x in xs { total += x }\n\
         \x20   println(\"total = {total}\")\n\
         \x20   println(\"len = {xs.len()}\")\n\
         \x20   println(3.0 / 2.0)\n\
         }\n",
    )
    .unwrap();

    let (ok, out, err) = run_mode("repl", path.to_str().unwrap());
    assert!(ok, "REPL run failed\nstderr:\n{err}");
    assert_eq!(out, "fib(10) = 55\ntotal = 108\nlen = 6\n1.5\n");
}

#[test]
fn interactive_repl_executes_statements_and_echoes_expressions() {
    // Drive an interactive session over stdin: a binding then a statement using it,
    // a function definition then a call to it, and a bare expression that is echoed
    // by the REPL wrapping it in `println`. Prompts go to stderr, so stdout is
    // exactly the three printed results.
    let mut child = Command::new(env!("CARGO_BIN_EXE_prepoly"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn prepoly repl");

    {
        let mut stdin = child.stdin.take().unwrap();
        stdin
            .write_all(
                b"let x = 21\n\
                  println(x * 2)\n\
                  fun sq(n: int32) -> int32 { return n * n }\n\
                  println(sq(5))\n\
                  6 * 7\n",
            )
            .unwrap();
    } // dropping stdin signals EOF, ending the session

    let out = child.wait_with_output().expect("wait for prepoly repl");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout, "42\n25\n42\n", "interactive REPL stdout");
}
