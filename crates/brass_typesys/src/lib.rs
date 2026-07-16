//! The single authority for RELATIONS between Brass types: numeric flow
//! (which implicit conversions exist), the common-type lattice of binary
//! operators, and structural record compatibility/satisfaction.
//!
//! Every pass that needs to DECIDE a relation (the hm and infer checker
//! passes, monomorphization, both back ends) asks this crate; none keeps a
//! private copy. Per-type utilities with no second type involved
//! (`peel_modes`, `is_fully_known`, ...) stay in `brass_hir`.

pub mod convert;
pub mod defaults;
pub mod flow;
pub mod rows;
pub mod specialize;
pub mod structural;
pub mod valueflow;

pub use defaults::default_constructible;
pub use flow::{Flow, common_numeric_type, numeric_flow, numeric_flows_into};
pub use rows::{
    ParamRow, Presence, Row, RowField, RowInfo, RowTy, check_row, field_satisfies, view_type,
};
pub use specialize::{Generated, KeyedNeed, mangled_name, specialize_all};
pub use structural::{
    function_part_compatible, record_satisfies, record_satisfies_fields, signature_satisfies,
    types_compatible, types_invariant,
};
pub use valueflow::{flow_probe, flow_unify, strip_nullable};
