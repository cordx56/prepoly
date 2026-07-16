#!/usr/bin/env bash
# Build the compiler and run every example, printing its output.
set -euo pipefail

cd "$(dirname "$0")"
cargo build --quiet -p brass_driver
BIN=./target/debug/brass

examples=(
    examples/01_records.cz
    examples/02_sum_types.cz
    examples/03_interfaces.cz
    examples/04_sum_interface.cz
    examples/05_nullable_and_result.cz
    examples/06_structural_subtyping.cz
    examples/07_closures.cz
    examples/08_pattern_matching.cz
    examples/09_collections.cz
    examples/10_strings_and_conversions.cz
    examples/11_control_flow.cz
    examples/12_concurrency.cz
    examples/13_file_io.cz
    examples/14_type_safety.cz
    examples/15_numeric_conversions.cz
    examples/16_method_inference.cz
    examples/17_higher_order.cz
    examples/18_methods.cz
    examples/modules/main.cz
)

for ex in "${examples[@]}"; do
    echo "===== $ex ====="
    "$BIN" "$ex"
    echo
done
