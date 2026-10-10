//! Independent certified arithmetic for observed dynamic production candidates.
//! Optional validation and failure diagnostics remain separate from sampling.
mod checker;
pub(in crate::kernel) mod failure;
mod primitives;
mod radius;
pub(in crate::kernel) mod runtime;
pub(in crate::kernel) mod validation;

pub(in crate::kernel) mod binding;
