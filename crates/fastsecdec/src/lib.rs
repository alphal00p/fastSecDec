//! Native sector integration built on HEPKit graphs and Symbolica expressions.
//!
//! Graphs, momentum routing and algebra retain their existing ecosystem types.
//! This crate owns only integral-specific normalization and generation state.

#![forbid(unsafe_code)]

pub mod error;
pub mod generation;
pub mod input;
pub mod integration;
pub mod kernel;
pub mod parametric;
pub mod status;

pub use error::{Error, Result};
pub use feynkit_graph::{EdgeId, FeynmanDiagram, IntegralFamily};
pub use feynkit_kinematics::Kinematics;
pub use feynkit_model::{Model, ParameterCard};
pub use symbolica::atom::{Atom, AtomCore, Symbol};
