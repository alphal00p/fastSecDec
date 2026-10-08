//! Deep inspection of one independently saved native record.
use crate::{CliResult, artifact::Artifact};
use fastsecdec::kernel::{KernelLoadOptions, PortableMetadata, indexed::IndexedReader};
use std::{fs::File, path::Path};

pub(super) fn inspect(
    path: &Path,
    artifact: Artifact,
    options: KernelLoadOptions,
    id: usize,
    expressions: bool,
    plain: bool,
    json: bool,
) -> CliResult<()> {
    let catalogue = artifact.catalogue().ok_or("missing indexed catalogue")?;
    let descriptor = catalogue.sector(id)?;
    let mut reader = IndexedReader::new(
        File::open(artifact.data_path(path)?)?,
        catalogue.clone(),
        options,
    )?;
    let kernels = reader.load_sector(id)?;
    let sources = &descriptor.receipt.source_indices;
    if json {
        let mut charts = kernels
            .generation_metadata()
            .map(|metadata| serde_json::to_value(PortableMetadata::charts_for_sector(metadata, 0)))
            .transpose()?;
        if let Some(serde_json::Value::Array(charts)) = &mut charts {
            for chart in charts {
                for field in ["source_index", "representative"] {
                    let local = chart[field]
                        .as_u64()
                        .ok_or("invalid chart presentation index")?;
                    let global = sources
                        .get(usize::try_from(local)?)
                        .ok_or("chart presentation index exceeds record mapping")?;
                    chart[field] = (*global).into();
                }
                chart["kernel_sector"] = id.into();
            }
        }
        crate::report(
            &serde_json::json!({
                "content_id":artifact.content_id,"inspection_mode":"deep",
                "binary_loaded":true,"loaded_sectors":1,
                "binary_validated":options.validate,"validation_scope":"selected_sector_record",
                "metadata_identity_validated":artifact.validation.metadata_identity,
                "generation":artifact.generation,"generation_timings":artifact.generation_timings,
                "sectors":catalogue.sector_count(),"orders":catalogue.orders,"components":catalogue.components,
                "runtime_parameters":catalogue.runtime_parameters,"parameters_bound":kernels.parameters_bound(),
                "selected_sector":{
                    "id":id,"content_id":kernels.sector_content_id(0)?,
                    "dimension":kernels.sectors()[0].dimension(),
                    "local_orders":kernels.orders(),"local_components":kernels.components(),
                    "global_output_indices":descriptor.output_indices,
                    "source_chart_generation_modes":super::source_chart_modes_mapped(&artifact,&kernels,0,Some(sources)),
                    "evaluator_statistics":kernels.sectors()[0].statistics(),"charts":charts,
                }
            }),
            true,
        )?;
    } else {
        let (width, colors) = super::presentation_options(plain);
        print!(
            "{}",
            super::presentation::render_record(
                path,
                &artifact,
                &kernels,
                id,
                sources,
                expressions,
                width,
                colors
            )?
        );
    }
    Ok(())
}
