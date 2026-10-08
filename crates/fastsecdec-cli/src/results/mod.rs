//! Numerical result transport and presentation; result-only paths never load kernels.
mod display;
mod storage;

use std::path::Path;

use clap::{Args, ValueEnum};
use fastsecdec::{
    reference::CoefficientKey,
    results::{ResultReferenceSelection, ResultSectorSort, read_result},
    status::CoefficientComponent,
};

use crate::CliResult;

pub use storage::assemble_manifest;

pub fn check_destination(
    path: &Path,
    artifact: &crate::artifact::Artifact,
    artifact_path: &Path,
    checkpoint: &Path,
    reference: Option<&crate::reference::PreparedReference>,
) -> CliResult<()> {
    let (metadata, data) = crate::artifact::paths(artifact_path)?;
    let mut protected = vec![metadata, data, checkpoint.to_path_buf()];
    protected.push(artifact.data_path(artifact_path)?);
    protected.extend(artifact.source_paths());
    if let Some(reference) = reference {
        protected.push(reference.settings.path.clone());
    }
    storage::protect_output(path, protected.iter().map(|path| path.as_path()))
}

pub fn save(path: &Path, result: &fastsecdec::results::SavedIntegrationResult) -> CliResult<()> {
    crate::artifact::atomic_write(path, &fastsecdec::results::encode_result(result)?)
}

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Sort {
    #[default]
    Id,
    Magnitude,
    Error,
}

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Component {
    #[default]
    Real,
    Imag,
}

#[derive(Args, Default)]
pub struct ViewArgs {
    #[arg(long, value_enum, default_value = "id")]
    sort: Sort,
    /// Laurent order used for magnitude/error sorting.
    #[arg(long, allow_hyphen_values = true)]
    order: Option<i32>,
    #[arg(long, value_enum, default_value = "real")]
    component: Component,
}

impl ViewArgs {
    fn ordering(&self) -> CliResult<ResultSectorSort> {
        if matches!(self.sort, Sort::Id) {
            return Ok(ResultSectorSort::Id);
        }
        let key = CoefficientKey {
            order: self
                .order
                .ok_or("magnitude/error sorting requires --order")?,
            component: match self.component {
                Component::Real => CoefficientComponent::Real,
                Component::Imag => CoefficientComponent::Imag,
            },
        };
        Ok(match self.sort {
            Sort::Id => ResultSectorSort::Id,
            Sort::Magnitude => ResultSectorSort::Magnitude(key),
            Sort::Error => ResultSectorSort::StandardError(key),
        })
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ReferenceSource {
    Estimate,
    Stored,
}

pub fn show(path: &Path, options: &ViewArgs, json: bool) -> CliResult<()> {
    let result = read_result(&std::fs::read(path)?)?;
    display::show(&result, options.ordering()?, json)
}

pub fn export_reference(path: &Path, output: &Path, source: ReferenceSource) -> CliResult<()> {
    let result = read_result(&std::fs::read(path)?)?;
    let reference = result.reference(match source {
        ReferenceSource::Estimate => ResultReferenceSelection::Estimate,
        ReferenceSource::Stored => ResultReferenceSelection::StoredReference,
    })?;
    storage::protect_output(output, [path])?;
    crate::artifact::atomic_write(
        output,
        &fastsecdec::reference::encode_reference(&reference)?,
    )
}
