use super::super::{
    DiscoveredSector, Preparation, PreparedChartSource, StreamingError,
    codec::{self, invalid},
    prepare, records,
};
use crate::generation::GenerationProgress;
use std::{ops::ControlFlow, path::Path};

/// Apply one selected recipe to the complete shared chart. Its exact symmetry,
/// formula key and output record belong exclusively to that recipe's context.
pub fn discover_prepared(
    root: &Path,
    preparation: &Preparation,
    source: &PreparedChartSource,
    progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<DiscoveredSector, StreamingError> {
    if source.source_identity != preparation.source_identity
        || source.dimension != preparation.dimension
        || preparation
            .charts
            .get(source.index)
            .is_none_or(|job| job.index != source.index || job.map != source.map)
    {
        return Err(invalid("foreign prepared chart source receipt"));
    }
    let context = records::read_source(root, &preparation.source)?;
    let data = records::read_prepared(root, &source.record)?;
    let (map, _, _): (records::Map, _, _) = codec::read(root, &source.map, "map")?;
    let map = map.native()?;
    if data.index != source.index
        || data.source_identity != source.source_identity
        || data.map_reference != source.map
        || data.map != map
        || data.parameters != context.targets
        || data.parameters.len() != preparation.dimension
        || (context.options.contour_enabled() && !data.retains_declarations)
    {
        return Err(invalid(
            "prepared chart source identity, map or declaration mismatch",
        ));
    }
    prepare::discover_loaded(
        root,
        preparation,
        context,
        map,
        source.index,
        Some(data),
        progress,
    )
}
