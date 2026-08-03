#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::too_many_arguments)]

// Progenitor's request examples are documentation-only fragments; the generated
// client is validated through atla-core's transport and contract tests instead.
#[cfg(not(doctest))]
include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
