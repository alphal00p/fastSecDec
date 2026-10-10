use crate::CliResult;
use fastsecdec::{
    Atom,
    contour::{ContourDiagnosticsMode, ContourJacobian, ContourSettings, ContourValidation},
    kernel::{KernelLoadOptions, KernelSet, ProgramRecipe, indexed::ProgramArchiveReader},
    results::{ExactContributionPolicy, KernelResultManifest, ResultScope},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
use symbolica::{
    atom::{AtomView, Symbol},
    parser::ParseSettings,
};

#[path = "diagnose.rs"]
mod diagnose;
#[path = "partition.rs"]
mod partition;
#[path = "qmc.rs"]
mod qmc;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Pin {
    pub path: PathBuf,
    pub blake3: String,
}
impl Pin {
    pub fn new(path: &Path) -> CliResult<Self> {
        let path = path.canonicalize()?;
        Ok(Self {
            blake3: blake3::hash(&fs::read(&path)?).to_hex().to_string(),
            path,
        })
    }
    pub fn verify(&self) -> CliResult<()> {
        let mut reader = File::open(&self.path)?;
        let mut hash = blake3::Hasher::new();
        hash.update_reader(&mut reader)?;
        require(
            hash.finalize().to_hex().as_str() == self.blake3,
            "pinned input changed",
        )
    }
}
#[derive(Deserialize)]
#[serde(tag = "storage", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Input {
    Flat {
        saved: Pin,
    },
    Indexed {
        manifest: Pin,
        data: Pin,
        recipe: ProgramRecipe,
        content_id: String,
        source_identity: String,
    },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Arm {
    pub id: String,
    pub settings: ContourSettings,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plan {
    pub input: Input,
    pub expected_recipe: ProgramRecipe,
    pub expected_jacobian: ContourJacobian,
    pub expected_sectors: usize,
    pub parameters: BTreeMap<String, f64>,
    pub arms: Vec<Arm>,
    pub sector_ids: Option<Vec<u64>>,
    pub seed: u64,
    pub pilot_seed: u64,
    pub points: u64,
    pub shifts: u32,
    pub top_k_per_sector: usize,
    pub setup_seconds: f64,
    pub sampling_seconds: f64,
}
pub(super) fn require(ok: bool, message: &str) -> CliResult<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
pub(super) fn save(path: &Path, value: &impl Serialize) -> CliResult<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut f, value)?;
    f.write_all(b"\n")?;
    Ok(())
}
pub(super) fn deadline(start: Instant, seconds: f64) -> CliResult<()> {
    require(
        start.elapsed().as_secs_f64() < seconds,
        "native cap-study stage deadline reached",
    )
}
pub(super) fn physics(plan: &Plan) -> CliResult<BTreeMap<Symbol, f64>> {
    let mut values = BTreeMap::new();
    for (name, value) in &plan.parameters {
        require(value.is_finite(), "nonfinite physical input")?;
        let atom = Atom::parse(name, "feynkit_graph", ParseSettings::default())?;
        let AtomView::Var(var) = atom.as_view() else {
            return Err("non-symbol physical input".into());
        };
        require(
            values.insert(var.get_symbol(), *value).is_none(),
            "duplicate native physical input",
        )?;
    }
    Ok(values)
}
fn load(plan: &Plan) -> CliResult<(KernelSet, Value)> {
    let start = Instant::now();
    let (owner, identity) = match &plan.input {
        Input::Flat { saved } => {
            saved.verify()?;
            let metadata: Value = serde_json::from_reader(File::open(&saved.path)?)?;
            require(
                metadata["endpoint_mode"] == "symbolic",
                "flat owner endpoint mode differs",
            )?;
            let pin: Pin = serde_json::from_value(metadata["artifact"].clone())?;
            pin.verify()?;
            let owner = KernelSet::from_bytes(&fs::read(&pin.path)?)?;
            require(
                Some(owner.content_id()) == metadata["content_id"].as_str(),
                "flat owner content identity differs",
            )?;
            (
                owner,
                json!({"storage":"flat","saved":saved,"metadata":metadata}),
            )
        }
        Input::Indexed {
            manifest,
            data,
            recipe,
            content_id,
            source_identity,
        } => {
            manifest.verify()?;
            let producer: Value = serde_json::from_reader(File::open(&manifest.path)?)?;
            require(
                producer["generation"]["mode"] == "symbolic"
                    && producer["generation"]["subtraction"] == "integrate_by_parts",
                "indexed owner is not Symbolic endpoint IBP",
            )?;
            data.verify()?;
            let mut reader = ProgramArchiveReader::from_reader(
                File::open(&data.path)?,
                KernelLoadOptions { validate: false },
            )?;
            require(
                reader.catalogue().source_identity.as_deref() == Some(source_identity.as_str()),
                "indexed source identity differs",
            )?;
            let catalogue = serde_json::to_value(reader.catalogue())?;
            require(
                producer["programs"]["catalogue"] == catalogue,
                "pinned producer/native catalogue differ",
            )?;
            let mut selected = reader.select(*recipe)?;
            require(
                selected.catalogue().content_id == *content_id,
                "indexed recipe identity differs",
            )?;
            (
                selected.load_all()?,
                json!({"storage":"indexed","manifest":manifest,"data":data,"catalogue":catalogue,"selected_recipe":recipe}),
            )
        }
    };
    require(
        owner.program_recipe() == plan.expected_recipe,
        "loaded recipe differs",
    )?;
    require(
        owner.compilation_settings().contour_jacobian == plan.expected_jacobian,
        "loaded Jacobian policy differs",
    )?;
    require(
        owner.sectors().len() == plan.expected_sectors,
        "loaded residual inventory differs",
    )?;
    Ok((
        owner,
        json!({"identity":identity,"restore_and_integrity_seconds":start.elapsed().as_secs_f64()}),
    ))
}
pub(super) fn scope(owner: &KernelSet, plan: &Plan) -> CliResult<ResultScope> {
    let manifest = KernelResultManifest::from_kernels(owner);
    match &plan.sector_ids {
        None => Ok(ResultScope::FullIntegral),
        Some(ids) => Ok(manifest.canonical_scope(&ResultScope::SelectedSectors {
            sector_ids: ids.clone(),
            exact_policy: ExactContributionPolicy::ExcludeAll,
        })?),
    }
}
pub(super) fn bound_scope(
    owner: &mut KernelSet,
    plan: &Plan,
    parameters: &BTreeMap<Symbol, f64>,
) -> CliResult<ResultScope> {
    // Native scope admission validates exact coefficients as well as sectors.
    // Bind runtime parameters before asking it to canonicalize a selection.
    if plan.sector_ids.is_some() {
        let first = plan.arms.first().ok_or("empty cap-study plan")?;
        owner.bind_parameters_with_contour(parameters, &first.settings)?;
    }
    scope(owner, plan)
}
pub(super) fn admit(
    owner: &mut KernelSet,
    plan: &Plan,
    arm: &Arm,
    values: &BTreeMap<Symbol, f64>,
    scope: &ResultScope,
    setup_allowance: f64,
) -> CliResult<Value> {
    require(
        arm.settings.validation.policy == ContourValidation::Pilot
            && arm.settings.validation.pilot_points == 16,
        "cap study requires actual Pilot16",
    )?;
    let start = Instant::now();
    owner.bind_parameters_with_contour(values, &arm.settings)?;
    let bind_seconds = start.elapsed().as_secs_f64();
    deadline(start, setup_allowance)?;
    let pilot = Instant::now();
    let report =
        crate::contour_pilot::run(owner, &arm.settings, plan.pilot_seed, Some(scope), |_| {
            deadline(start, setup_allowance)
        })?;
    owner.validate_integration_readiness(scope)?;
    Ok(
        json!({"bind_seconds":bind_seconds,"pilot_seconds":pilot.elapsed().as_secs_f64(),"report":report,
        "provenance":report.as_ref().map(|r|crate::contour_pilot::provenance(owner,plan.pilot_seed,r))}),
    )
}
pub(super) fn execute() -> CliResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    require(
        args.len() == 3 && (args[0] == "sample" || args[0] == "diagnose"),
        "usage: contour_cap_study {sample|diagnose} PLAN_JSON OUT",
    )?;
    let plan_path = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    fs::create_dir(out)?;
    if args[0] == "diagnose" {
        return diagnose::run(plan_path, out);
    }
    let plan: Plan = serde_json::from_reader(File::open(plan_path)?)?;
    require(
        !plan.arms.is_empty() && plan.expected_sectors > 0 && plan.top_k_per_sector > 0,
        "empty cap-study plan",
    )?;
    require(
        plan.setup_seconds.is_finite()
            && plan.setup_seconds > 0.
            && plan.sampling_seconds.is_finite()
            && plan.sampling_seconds > 0.,
        "invalid stage limits",
    )?;
    let unique = plan
        .arms
        .iter()
        .map(|a| &a.id)
        .collect::<std::collections::BTreeSet<_>>();
    require(unique.len() == plan.arms.len(), "duplicate arm IDs")?;
    save(
        &out.join("invocation.json"),
        &json!({"argv":args,"plan":Pin::new(plan_path)?,"executable":Pin::new(&std::env::current_exe()?)?}),
    )?;
    let load_start = Instant::now();
    let (mut owner, loading) = load(&plan)?;
    deadline(load_start, plan.setup_seconds)?;
    let parameters = physics(&plan)?;
    let scope = bound_scope(&mut owner, &plan, &parameters)?;
    owner.set_contour_diagnostics(ContourDiagnosticsMode::Disabled)?;
    save(
        &out.join("setup.json"),
        &json!({"loading":loading,"scope":scope,"compilation":owner.compilation_settings(),"orders":owner.orders(),"components":owner.components(),"selected_ids":plan.sector_ids,"diagnostics":"disabled during sampling"}),
    )?;
    qmc::run(
        &plan,
        &mut owner,
        &parameters,
        &scope,
        out,
        load_start.elapsed().as_secs_f64(),
    )
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
