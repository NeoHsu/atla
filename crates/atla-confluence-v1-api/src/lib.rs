#![allow(clippy::doc_lazy_continuation)]

// Progenitor's request examples are documentation-only fragments, and some contain
// intentionally raw URLs/JSON that rustdoc cannot compile as Rust examples. The
// hand-written atla-core adapters are covered by their own contract tests.
#[cfg(not(doctest))]
include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
