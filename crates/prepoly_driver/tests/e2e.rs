//! End-to-end tests for record-field and sum-variant features. Each `*.pp` under
//! `e2e_tests/` is run through the driver and its stdout compared byte-for-byte to
//! a sibling `*.out`. Cases are grouped by subsystem: `e2e_tests/field/` (record
//! and struct fields) and `e2e_tests/variant/` (sum types, variant construction,
//! pattern matching, and variant-field access). Several cases pin behaviors that
//! previously miscompiled (variant-qualified field binding, a sum common field at
//! differing per-variant offsets, and nested refutable sub-patterns).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn e2e_root() -> PathBuf {
    // The driver manifest lives at crates/prepoly_driver; the repo root is two up.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../e2e_tests")
}

/// Every `*.pp` under `dir`, recursively, in sorted order (so failures are stable).
fn collect_cases(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_cases(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "pp") {
            out.push(path);
        }
    }
}

#[test]
fn e2e_cases_produce_expected_output() {
    let bin = env!("CARGO_BIN_EXE_prepoly");
    let root = e2e_root();
    let mut cases = Vec::new();
    collect_cases(&root, &mut cases);
    assert!(
        !cases.is_empty(),
        "no e2e `.pp` cases found under {}",
        root.display()
    );

    for pp in &cases {
        let expected = fs::read_to_string(pp.with_extension("out")).unwrap_or_else(|e| {
            panic!(
                "missing/unreadable expected output for {}: {e}",
                pp.display()
            )
        });
        let out = Command::new(bin)
            .arg(pp)
            .output()
            .expect("spawn prepoly");
        assert!(
            out.status.success(),
            "{} failed to run (status {:?})\nstderr:\n{}",
            pp.display(),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr),
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            expected,
            "{} produced unexpected stdout",
            pp.display(),
        );
    }
}
