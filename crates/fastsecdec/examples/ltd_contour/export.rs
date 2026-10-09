//! Native graph/kinematics run cards; no generation is started by this exporter.
use super::{
    Result,
    import::{Imported, measure},
    records::Record,
};
use fastsecdec::{Atom, AtomCore, FeynmanDiagram, input::GraphIntegral};
use std::{fmt::Write, path::Path};
use symbolica::symbol;

pub fn write(root: &Path, record: &Record, imported: &Imported) -> Result<()> {
    let directory = root.join(record.slug());
    std::fs::create_dir_all(&directory)?;
    // The public HEPKit serializer remains the DOT owner. Verify its exact
    // denominator round trip before making the input available to the CLI.
    let dot = imported.graph.diagram().to_dot()?;
    let reloaded = FeynmanDiagram::from_dot(imported.graph.diagram().model_arc(), &dot)?;
    let graph = GraphIntegral::new(
        std::sync::Arc::new(reloaded),
        imported.graph.family().kinematics(),
    )?
    .with_scalar_values(&std::collections::BTreeMap::from([(
        symbol!("UFO::mt"),
        record.mass(),
    )]))?;
    if graph.family().denominators() != imported.graph.family().denominators() {
        return Err("native DOT export changed the ordered denominator family".into());
    }
    std::fs::write(directory.join("graph.dot"), dot)?;
    let mut card = format!(
        "# Native import of {} from arXiv:1912.09291v2.\n# All-outgoing external vectors, exact native conservation; see validation.json.\n[input]\ngraph = \"graph.dot\"\nmodel = \"../model.json\"\n\n[kinematics]\nproducts = [\n",
        record.name
    );
    let dependent = imported.summary.dependent_external_edge;
    for (i, p) in imported
        .vectors
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != dependent)
    {
        for (j, q) in imported
            .vectors
            .iter()
            .enumerate()
            .skip(i)
            .filter(|(j, _)| *j != dependent)
        {
            writeln!(
                card,
                "  {{ left = {i}, right = {j}, value = {} }},",
                serde_json::to_string(&p.dot(q).to_canonical_string())?
            )?;
        }
    }
    let epsilon = Atom::var(symbol!("ltd::eps"));
    writeln!(
        card,
        "]\n\n[parameters]\n\"UFO::mt\" = {}\n\n[integral]\nregulator = \"ltd::eps\"\ndimension = \"4-2*ltd::eps\"\nmeasure_multiplier = {}\n\n[generation]\norder = 0\ncontour = true\n\n[integration]\npoints = 1024\nshifts = 8\nworkers = 8\nperiodization = \"korobov3\"\nrelative_tolerance = 0.001\naccuracy_target = {{ target = \"laurent_order\", order = 0 }}\n\n[integration.contour.deformation]\nmode = \"fixed\"\nlambda = 0.001",
        serde_json::to_string(&record.mass().to_canonical_string())?,
        serde_json::to_string(&measure(record.n_loops, epsilon).to_canonical_string())?
    )?;
    std::fs::write(directory.join("run.toml"), card)?;
    std::fs::write(
        directory.join("validation.json"),
        serde_json::to_string_pretty(&imported.summary)? + "\n",
    )?;
    Ok(())
}
