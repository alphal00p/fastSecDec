use super::{CliResult, fixtures};
use fastsecdec::{
    contour::{
        ContourJacobian, ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction,
    },
    kernel::{KernelSet, ProgramRecipe},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

#[path = "generate.rs"]
mod generate;
#[path = "sampling.rs"]
mod sampling;

pub(super) const CAPS: [f64; 4] = [0.1, 0.03, 0.01, 0.003];
pub(super) const EPOCHS: [u64; 3] = [1024, 2048, 4096];

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Pin {
    path: PathBuf,
    blake3: String,
}
impl Pin {
    pub fn new(path: &Path) -> CliResult<Self> {
        let path = path.canonicalize()?;
        Ok(Self {
            blake3: blake3::hash(&fs::read(&path)?).to_hex().to_string(),
            path,
        })
    }
    pub fn read(&self) -> CliResult<Vec<u8>> {
        let bytes = fs::read(&self.path)?;
        require(
            blake3::hash(&bytes).to_hex().as_str() == self.blake3,
            "pinned file changed",
        )?;
        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Saved {
    pub case: String,
    pub case_index: usize,
    pub recipe: ProgramRecipe,
    pub jacobian: ContourJacobian,
    pub artifact: Pin,
    pub content_id: String,
    pub endpoint_mode: String,
    pub schema: Value,
    pub reference: fixtures::Reference,
    pub producer: Value,
}

pub(super) fn require(condition: bool, message: &str) -> CliResult<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub(super) fn save(path: &Path, value: &impl Serialize) -> CliResult<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}
pub(super) fn event(out: &Path, value: Value) -> CliResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(out.join("events.jsonl"))?;
    serde_json::to_writer(&mut file, &value)?;
    file.write_all(b"\n")?;
    Ok(())
}
pub(super) fn deadline(start: Instant, seconds: f64) -> CliResult<()> {
    require(
        start.elapsed().as_secs_f64() < seconds,
        "bounded stage deadline reached",
    )
}
pub(super) fn schema(kernels: &KernelSet) -> CliResult<Value> {
    let metadata = kernels
        .generation_metadata()
        .ok_or("missing native generation metadata")?;
    Ok(json!({
        "orders": kernels.orders(), "components": kernels.components(),
        "residual_dimensions": kernels.sectors().iter().map(|s|s.dimension()).collect::<Vec<_>>(),
        "charts": metadata.charts().iter().map(|chart|json!({
            "source": chart.source_index(), "representative": chart.representative(),
            "permutation": chart.representative_permutation(), "sector": chart.kernel_sector(),
            "coordinates": chart.coordinates().target_parameters().iter().map(|s|s.to_string()).collect::<Vec<_>>(),
            "images": chart.coordinates().images().iter().map(ToString::to_string).collect::<Vec<_>>(),
            "measure": chart.coordinates().measure_jacobian().to_string(),
        })).collect::<Vec<_>>()
    }))
}
pub(super) fn load(saved_path: &Path) -> CliResult<(Saved, KernelSet, Value)> {
    let start = Instant::now();
    let saved: Saved = serde_json::from_slice(&fs::read(saved_path)?)?;
    require(
        saved.endpoint_mode == "symbolic",
        "endpoint mode is not Symbolic",
    )?;
    let bytes = saved.artifact.read()?;
    let read_seconds = start.elapsed().as_secs_f64();
    let restore = Instant::now();
    let kernels = KernelSet::from_bytes(&bytes)?;
    require(
        kernels.content_id() == saved.content_id,
        "native content identity changed",
    )?;
    require(
        kernels.program_recipe() == saved.recipe,
        "native recipe changed",
    )?;
    require(
        kernels.compilation_settings().contour_jacobian == saved.jacobian,
        "native Jacobian policy changed",
    )?;
    require(
        schema(&kernels)? == saved.schema,
        "native restored coordinate/layout schema changed",
    )?;
    require(
        kernels
            .runtime_parameters()
            .iter()
            .all(|p| fastsecdec::contour::is_contour_parameter(*p)),
        "fixture contains unbound physical inputs",
    )?;
    let details = json!({"read_and_integrity_seconds":read_seconds,
        "restore_seconds":restore.elapsed().as_secs_f64(),
        "primary_cache": kernels.sectors().iter().map(|s|s.primary_evaluator_restoration()).collect::<Vec<_>>(),
        "restored_statistics":kernels.sectors().iter().map(|s|s.statistics()).collect::<Vec<_>>(),
        "compilation_settings":kernels.compilation_settings()});
    Ok((saved, kernels, details))
}
pub(super) fn settings(recipe: ProgramRecipe, cap: f64) -> CliResult<ContourSettings> {
    require(
        CAPS.contains(&cap),
        "cap is outside the frozen admission ladder",
    )?;
    let deformation = match recipe {
        ProgramRecipe::FixedV1 => ContourMode::Fixed { lambda: cap },
        ProgramRecipe::DynamicPolynomialV1 | ProgramRecipe::DynamicSignAwareV1 => {
            ContourMode::Dynamical {
                safety_fraction: 0.8,
                lambda_cap: cap,
                displacement_cap: 1.,
                construction: if recipe == ProgramRecipe::DynamicPolynomialV1 {
                    DynamicConstruction::Polynomial
                } else {
                    DynamicConstruction::SignAware
                },
            }
        }
        _ => return Err("undeformed is not a scalar campaign arm".into()),
    };
    Ok(ContourSettings {
        deformation,
        validation: ContourValidationOptions {
            policy: ContourValidation::Pilot,
            pilot_points: 16,
        },
    })
}
fn recipe(name: &str) -> CliResult<ProgramRecipe> {
    match name {
        "fixed" => Ok(ProgramRecipe::FixedV1),
        "polynomial" => Ok(ProgramRecipe::DynamicPolynomialV1),
        "sign-aware" => Ok(ProgramRecipe::DynamicSignAwareV1),
        _ => Err("expected fixed, polynomial or sign-aware".into()),
    }
}
fn jacobian(name: &str) -> CliResult<ContourJacobian> {
    match name {
        "symbolic" => Ok(ContourJacobian::Symbolic),
        "dual" => Ok(ContourJacobian::Dual),
        _ => Err("expected symbolic or dual".into()),
    }
}

pub(super) fn execute() -> CliResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let usage = "actions: prepare OUT | generate CASE {fixed|polynomial|sign-aware} {symbolic|dual} OUT [--horner-iterations N] | admit SAVED_JSON OUT | select OUT ADMISSION_JSON... (six) | sample SAVED_JSON COMMON_CAP_JSON SEED OUT";
    let out = match args.as_slice() {
        [a, o] if a == "prepare" => PathBuf::from(o),
        [a, _, _, _, o, ..] if a == "generate" => PathBuf::from(o),
        [a, _, _, _, o] if a == "sample" => PathBuf::from(o),
        [a, _, o] if a == "admit" => PathBuf::from(o),
        [a, o, rest @ ..] if a == "select" && rest.len() == 6 => PathBuf::from(o),
        _ => return Err(usage.into()),
    };
    let compilation = if args[0] == "generate" {
        Some(generate::compilation_settings(
            jacobian(&args[3])?,
            &args[5..],
        )?)
    } else {
        None
    };
    fs::create_dir(&out)?;
    save(
        &out.join("invocation.json"),
        &json!({"argv":args,"executable":Pin::new(&std::env::current_exe()?)?,"protocol":"scalar-symbolic-endpoints-v1"}),
    )?;
    let result = match args[0].as_str() {
        "prepare" => generate::prepare(&out),
        "generate" => generate::generate(
            &args[1],
            recipe(&args[2])?,
            jacobian(&args[3])?,
            &out,
            compilation.expect("validated generation settings"),
        ),
        "admit" => sampling::admit(Path::new(&args[1]), &out),
        "select" => sampling::select(&args[2..], &out),
        "sample" => sampling::sample(
            Path::new(&args[1]),
            Path::new(&args[2]),
            args[3].parse()?,
            &out,
        ),
        _ => unreachable!(),
    };
    if let Err(error) = &result {
        save(
            &out.join("failure.json"),
            &json!({"error":error.to_string(),"accepted":false}),
        )?;
    }
    result
}
