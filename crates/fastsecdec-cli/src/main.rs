mod artifact;
mod config;
mod diagnostics;
mod display;
mod driver;
mod generate;
mod generation_report;
mod input;
mod inspect;
mod math_display;
mod reference;
mod results;
mod status_policy;
mod terminal_policy;

use clap::{Args, Parser, Subcommand};
use config::IntegrationInput;
use fastsecdec::AtomCore;
use std::{
    io::IsTerminal,
    path::PathBuf,
    process::{Command, ExitCode},
};
type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct ReportedFailure;
impl std::fmt::Display for ReportedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("diagnostic evaluations failed")
    }
}
impl std::error::Error for ReportedFailure {}

#[derive(Parser)]
#[command(
    name = "fastsecdec",
    version,
    about = "Native Feynman-parameter generation and numerical sector integration"
)]
struct Cli {
    #[arg(long, global = true, help = "Write the final report as JSON")]
    json: bool,
    #[arg(
        long,
        global = true,
        help = "Disable the live dashboard and report colors"
    )]
    plain: bool,
    #[arg(
        long,
        global = true,
        help = "Stream status snapshots as JSON lines to stderr"
    )]
    status_json: bool,
    /// Minimum interval for JSON coefficient/integration status; zero emits every update.
    #[arg(long, global = true, default_value_t = 100)]
    status_interval_ms: u64,
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Generate portable O2 kernels from a native TOML run card.
    Generate {
        input: PathBuf,
        /// Artifact basename, such as output/integral.fsd, without .json or .dat.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Caller-owned workers for geometry, symbolic generation, and compilation.
        #[arg(long = "workers", visible_alias = "geometry-workers", default_value_t = default_generation_workers())]
        geometry_workers: std::num::NonZeroUsize,
    },
    /// Generate and integrate a native TOML run card.
    Run {
        input: PathBuf,
        /// Artifact basename, such as output/integral.fsd, without .json or .dat.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Caller-owned generation workers; resumed artifacts need no generation.
        #[arg(long = "generation-workers", visible_alias = "geometry-workers", default_value_t = default_generation_workers())]
        geometry_workers: std::num::NonZeroUsize,
        #[command(flatten)]
        integration: IntegrationArgs,
    },
    /// Integrate a saved portable artifact, optionally resuming a checkpoint.
    Integrate {
        artifact: PathBuf,
        #[command(flatten)]
        integration: IntegrationArgs,
    },
    /// View a saved numerical result without loading graphs or compiled kernels.
    ShowResult {
        path: PathBuf,
        #[command(flatten)]
        view: results::ViewArgs,
    },
    /// Export an explicitly selected estimate or original stored reference.
    ExportReference {
        path: PathBuf,
        #[arg(long, value_enum)]
        source: results::ReferenceSource,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Inspect native input or an existing portable artifact.
    Inspect {
        path: PathBuf,
        /// Inspect one compiled sector by its zero-based ID from the overview.
        #[arg(long)]
        sector: Option<usize>,
        #[arg(long)]
        expressions: bool,
    },
    /// Measure repeated kernel evaluations with reproducible interior points.
    Benchmark {
        artifact: PathBuf,
        #[arg(long, default_value_t = 100_000)]
        points: usize,
        #[arg(long, default_value_t = 5)]
        repetitions: usize,
    },
    /// Sample bounded coordinate faces from inside, including face intersections.
    CheckBoundaries {
        artifact: PathBuf,
        #[arg(long, value_delimiter = ',', default_value = "3,6,9,12,15")]
        exponents: Vec<i32>,
        #[arg(long, default_value_t = 2)]
        max_codimension: usize,
        #[arg(long, default_value_t = 10_000)]
        max_probes: usize,
        /// Sampled growth threshold per approached axis; not an integrability test.
        #[arg(long, default_value_t = 0.5)]
        growth_tolerance: f64,
        /// Decreasing scales in (0,1), each relative to the original distances.
        #[arg(long, value_delimiter = ',')]
        retry_scales: Vec<f64>,
    },
}

impl Action {
    fn validate_generation_output(&self) -> CliResult<()> {
        if let Self::Generate { input, output, .. } | Self::Run { input, output, .. } = self {
            let output = output
                .clone()
                .unwrap_or_else(|| input::artifact_path(input));
            artifact::paths(&output)?;
        }
        Ok(())
    }
}

#[derive(Args, Default)]
struct IntegrationArgs {
    /// TOML file with [parameters] containing this integration point.
    #[arg(long)]
    parameters: Option<PathBuf>,
    /// Override one runtime scalar with NAME=VALUE; may be repeated.
    #[arg(long = "parameter", value_name = "NAME=VALUE")]
    parameter: Vec<String>,
    /// Clear a stored/card subset and integrate the complete parent integral.
    #[arg(long, conflicts_with_all = ["sectors", "exact_contributions"])]
    full_integral: bool,
    /// Compiled kernel IDs, comma-separated; use none for an explicit empty subset.
    #[arg(long, requires = "exact_contributions")]
    sectors: Option<String>,
    /// Include or exclude the complete folded exact offset for selected scope.
    #[arg(long, requires = "sectors", value_parser = ["include", "exclude"])]
    exact_contributions: Option<String>,
    #[arg(long)]
    method: Option<String>,
    #[arg(long)]
    points: Option<u64>,
    #[arg(long)]
    shifts: Option<u32>,
    /// Explicit published generating vector; the default remains kuo33002.
    #[arg(long, value_parser = ["kuo33002", "kuo38005", "kuo39101", "hkkn-alpha3"])]
    lattice: Option<String>,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long)]
    workers: Option<usize>,
    #[arg(long)]
    absolute_tolerance: Option<f64>,
    #[arg(long)]
    relative_tolerance: Option<f64>,
    #[arg(long)]
    checkpoint: Option<PathBuf>,
    #[arg(long)]
    resume: bool,
    /// Compare with a versioned reference JSON file; relative paths use cwd.
    #[arg(long)]
    reference: Option<PathBuf>,
    /// Save the accepted numerical result as a native versioned document.
    #[arg(long)]
    save_result: Option<PathBuf>,
}

impl IntegrationArgs {
    fn apply(&self, settings: &mut IntegrationInput) -> CliResult<()> {
        let mut parameter_values = std::collections::BTreeMap::new();
        if let Some(path) = &self.parameters {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Point {
                parameters: std::collections::BTreeMap<String, toml::Value>,
            }
            let point: Point = toml::from_str(&std::fs::read_to_string(path)?)?;
            parameter_values = point.parameters;
            settings.parameters.clear();
        }
        for value in &self.parameter {
            let (name, value) = value
                .split_once('=')
                .ok_or("--parameter requires NAME=VALUE")?;
            if name.trim().is_empty() {
                return Err("--parameter requires a nonempty name".into());
            }
            parameter_values.insert(
                name.trim().to_string(),
                toml::Value::String(value.to_string()),
            );
        }
        let mut supplied = std::collections::BTreeMap::new();
        for (name, value) in parameter_values {
            let symbol = input::symbol(&name)?;
            if supplied
                .insert(symbol, input::value_expression(&value)?)
                .is_some()
            {
                return Err(format!("duplicate runtime parameter {name}").into());
            }
        }
        let supplied = feynkit_model::resolve_scalar_bindings(supplied)?;
        for (symbol, expression) in supplied {
            let value = expression
                .evaluate(&std::collections::HashMap::<fastsecdec::Atom, f64>::new())
                .map_err(|error| {
                    format!(
                        "runtime parameter {} is not a real numeric expression: {error}",
                        fastsecdec::Atom::var(symbol).to_canonical_string()
                    )
                })?;
            if !value.is_finite() {
                return Err("runtime parameter values must be finite".into());
            }
            settings
                .parameters
                .insert(fastsecdec::Atom::var(symbol).to_canonical_string(), value);
        }

        if self.full_integral {
            settings.scope = fastsecdec::results::ResultScope::FullIntegral;
        }
        if let Some(ids) = &self.sectors {
            let sector_ids = if ids == "none" {
                Vec::new()
            } else {
                ids.split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<u64>, _>>()?
            };
            let exact_policy = match self.exact_contributions.as_deref() {
                Some("include") => fastsecdec::results::ExactContributionPolicy::IncludeAll,
                Some("exclude") => fastsecdec::results::ExactContributionPolicy::ExcludeAll,
                _ => {
                    return Err(
                        "sector selection requires an explicit exact-contributions policy".into(),
                    );
                }
            };
            settings.scope = fastsecdec::results::ResultScope::SelectedSectors {
                sector_ids,
                exact_policy,
            };
        }
        if let Some(value) = &self.method {
            settings.method = value.clone();
        }
        if let Some(value) = self.points {
            settings.points = value;
        }
        if let Some(value) = self.shifts {
            settings.shifts = value;
        }
        if let Some(value) = &self.lattice {
            settings.lattice = value.clone();
        }
        if let Some(value) = self.seed {
            settings.seed = value;
        }
        if let Some(value) = self.workers {
            settings.workers = value;
        }
        if let Some(value) = self.absolute_tolerance {
            settings.absolute_tolerance = value;
        }
        if let Some(value) = self.relative_tolerance {
            settings.relative_tolerance = value;
        }
        Ok(())
    }
}

fn default_generation_workers() -> std::num::NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN)
}

fn bind_parameters(
    kernels: &mut fastsecdec::kernel::KernelSet,
    settings: &IntegrationInput,
) -> CliResult<()> {
    let values = settings
        .parameters
        .iter()
        .map(|(name, value)| Ok((input::symbol(name)?, *value)))
        .collect::<CliResult<std::collections::BTreeMap<_, _>>>()?;
    kernels.bind_parameters(&values)?;
    Ok(())
}

fn main() -> ExitCode {
    // This documented Symbolica display setting never changes licensing. A
    // fresh process avoids changing global state after threads may exist.
    if std::env::var_os("SYMBOLICA_HIDE_BANNER").is_none() {
        let result = std::env::current_exe().and_then(|executable| {
            let mut command = Command::new(executable);
            command
                .args(std::env::args_os().skip(1))
                .env("SYMBOLICA_HIDE_BANNER", "1");
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                Err::<std::process::ExitStatus, _>(command.exec())
            }
            #[cfg(not(unix))]
            {
                command.status()
            }
        });
        return match result {
            Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
            Err(error) => {
                eprintln!("fastsecdec: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let cli = Cli::parse();
    // Check the public artifact name before run-card/reference I/O, terminal
    // setup or generation. In particular this also covers `run --resume`.
    let preflight = cli.command.validate_generation_output();
    let json = cli.json;
    let color =
        terminal_policy::ColorPolicy::for_stream(cli.plain, std::io::stderr().is_terminal());
    match preflight.and_then(|()| run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.is::<ReportedFailure>() {
                return ExitCode::FAILURE;
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({"error":{"message":error.to_string()}})
                );
            } else if color.enabled() {
                eprintln!("\x1b[31;1mfastsecdec\x1b[0m: {error}");
            } else {
                eprintln!("fastsecdec: {error}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> CliResult<()> {
    let render_json = cli.json;
    let make_dashboard = || {
        display::Dashboard::with_status_interval(
            !cli.plain && !cli.json,
            cli.status_json,
            cli.status_interval_ms,
        )
    };
    match cli.command {
        Action::Generate {
            input,
            output,
            geometry_workers,
        } => {
            let reference = reference::from_card(&input, None)?;
            let output = output.unwrap_or_else(|| input::artifact_path(&input));
            let mut dashboard = make_dashboard()?;
            let (artifact, kernels) = generate::generate_with_workers(
                &input,
                &output,
                &mut dashboard,
                reference.as_ref(),
                geometry_workers.get(),
            )?;
            if kernels.runtime_parameters().is_empty()
                && let Some(reference) = &reference
            {
                reference.validate_identity(kernels.content_id())?;
            }
            drop(dashboard);
            generation_report::print(&output, &artifact, &kernels, cli.plain, render_json)?;
        }
        Action::Run {
            input,
            output,
            geometry_workers,
            integration,
        } => {
            let reference = reference::from_card(&input, integration.reference.as_deref())?;
            let output = output.unwrap_or_else(|| input::artifact_path(&input));
            let mut dashboard = make_dashboard()?;
            let (artifact, mut kernels) = if integration.resume {
                artifact::Artifact::load_with_preflight(&output, |_| Ok(()))?
            } else {
                generate::generate_with_workers(
                    &input,
                    &output,
                    &mut dashboard,
                    reference.as_ref(),
                    geometry_workers.get(),
                )?
            };
            if integration.resume {
                artifact.verify_input_sources(&input)?;
            }
            let mut settings: IntegrationInput =
                serde_json::from_value(artifact.provenance.integration.clone())?;
            integration.apply(&mut settings)?;
            bind_parameters(&mut kernels, &settings)?;
            if let Some(reference) = &reference {
                reference.validate_identity(kernels.content_id())?;
            }
            settings.scope = fastsecdec::results::KernelResultManifest::from_kernels(&kernels)
                .canonical_scope(&settings.scope)?;
            let checkpoint = integration
                .checkpoint
                .unwrap_or_else(|| output.with_extension("checkpoint.json"));
            if let Some(path) = &integration.save_result {
                results::check_destination(
                    path,
                    &artifact,
                    &output,
                    &checkpoint,
                    reference.as_ref(),
                )?;
            }
            let result = driver::integrate(
                &artifact,
                &kernels,
                &settings,
                &checkpoint,
                integration.resume,
                &mut dashboard,
            )?;
            drop(dashboard);
            let saved =
                results::assemble(&artifact, &kernels, &settings, &result, reference.as_ref())?;
            if let Some(path) = &integration.save_result {
                results::save(path, &saved)?;
            }
            let comparison = reference
                .as_ref()
                .map(|reference| reference.report_saved(&saved))
                .transpose()?;
            integration_report(&result, comparison.as_ref(), render_json)?;
            if result.failed() {
                return Err(ReportedFailure.into());
            }
        }
        Action::Integrate {
            artifact: path,
            integration,
        } => {
            let mut reference = None;
            let (artifact, mut kernels) =
                artifact::Artifact::load_with_preflight(&path, |artifact| {
                    reference = reference::prepare(
                        artifact.resolved_reference(),
                        integration.reference.as_deref(),
                    )?;
                    Ok(())
                })?;
            let mut settings: IntegrationInput =
                serde_json::from_value(artifact.provenance.integration.clone())?;
            integration.apply(&mut settings)?;
            bind_parameters(&mut kernels, &settings)?;
            if let Some(reference) = &reference {
                reference.validate_identity(kernels.content_id())?;
            }
            settings.scope = fastsecdec::results::KernelResultManifest::from_kernels(&kernels)
                .canonical_scope(&settings.scope)?;
            let checkpoint = integration
                .checkpoint
                .unwrap_or_else(|| path.with_extension("checkpoint.json"));
            if let Some(result_path) = &integration.save_result {
                results::check_destination(
                    result_path,
                    &artifact,
                    &path,
                    &checkpoint,
                    reference.as_ref(),
                )?;
            }
            let mut dashboard = make_dashboard()?;
            let result = driver::integrate(
                &artifact,
                &kernels,
                &settings,
                &checkpoint,
                integration.resume,
                &mut dashboard,
            )?;
            drop(dashboard);
            let saved =
                results::assemble(&artifact, &kernels, &settings, &result, reference.as_ref())?;
            if let Some(path) = &integration.save_result {
                results::save(path, &saved)?;
            }
            let comparison = reference
                .as_ref()
                .map(|reference| reference.report_saved(&saved))
                .transpose()?;
            integration_report(&result, comparison.as_ref(), render_json)?;
            if result.failed() {
                return Err(ReportedFailure.into());
            }
        }
        Action::ShowResult { path, view } => results::show(&path, &view, render_json)?,
        Action::ExportReference {
            path,
            source,
            output,
        } => {
            results::export_reference(&path, &output, source)?;
            report(
                &serde_json::json!({"reference":artifact::relative_path(&output, std::path::Path::new("."))?}),
                render_json,
            )?;
        }
        Action::Inspect {
            path,
            expressions,
            sector,
        } => {
            if path
                .extension()
                .is_some_and(|extension| extension == "toml")
            {
                if sector.is_some() {
                    return Err(
                        "--sector requires a generated artifact basename, not a run card".into(),
                    );
                }
                let loaded = input::load(&path)?;
                let mut value = serde_json::json!({"name":loaded.label,"loops":loaded.loops,"parameters":loaded.propagators,
                    "domain":format!("{:?}",loaded.integrand.domain()),"terms":loaded.integrand.terms().len(),
                    "independent_externals":loaded.independent_externals,"dependent_externals":loaded.dependent_externals});
                if let Some(preparation) = loaded.family_preparation {
                    value["family_preparation"] = serde_json::to_value(preparation)?;
                    value["active_parameters"] = loaded.integrand.parameters().len().into();
                }
                if expressions {
                    value["density"] =
                        serde_json::Value::String(loaded.integrand.density().to_canonical_string());
                }
                report(&value, render_json)?;
            } else {
                inspect::artifact(&path, expressions, sector, cli.plain, render_json)?;
            }
        }
        Action::Benchmark {
            artifact,
            points,
            repetitions,
        } => {
            let (artifact, mut kernels) = artifact::Artifact::load(&artifact)?;
            let dashboard = display::Dashboard::new(false, false)?;
            let benchmark = fastsecdec::diagnostics::benchmark(
                &mut kernels,
                &fastsecdec::diagnostics::BenchmarkOptions {
                    points,
                    repetitions,
                    ..Default::default()
                },
                |progress| diagnostics::observe(&dashboard, cli.status_json, progress),
            )?;
            let failed =
                benchmark.stop == fastsecdec::diagnostics::DiagnosticStop::EvaluationFailure;
            let mut result = serde_json::to_value(benchmark)?;
            result["loading_seconds"] = artifact.loading_seconds.into();
            result["generation_timings"] = serde_json::to_value(artifact.generation_timings)?;
            report(&result, render_json)?;
            if failed {
                return Err(ReportedFailure.into());
            }
        }
        Action::CheckBoundaries {
            artifact,
            exponents,
            max_codimension,
            max_probes,
            growth_tolerance,
            retry_scales,
        } => {
            let (_, mut kernels) = artifact::Artifact::load(&artifact)?;
            let dashboard = display::Dashboard::new(false, false)?;
            let boundaries = fastsecdec::diagnostics::scan_boundaries(
                &mut kernels,
                &fastsecdec::diagnostics::BoundaryScanOptions {
                    sampling: fastsecdec::diagnostics::BoundaryOptions {
                        exponents,
                        max_codimension,
                        max_probes,
                        ..Default::default()
                    },
                    growth: fastsecdec::diagnostics::BoundaryGrowthOptions {
                        max_power_per_axis: growth_tolerance,
                        ..Default::default()
                    },
                    retry_scales,
                },
                |progress| diagnostics::observe_scan(&dashboard, cli.status_json, progress),
            )?;
            let failed = boundaries.diagnostics.failures;
            if render_json {
                println!("{}", serde_json::to_string_pretty(&boundaries)?);
            } else {
                diagnostics::display_scan(&boundaries, cli.plain);
            }
            if failed > 0 {
                return Err(ReportedFailure.into());
            }
        }
    }
    Ok(())
}

fn report(value: &serde_json::Value, json: bool) -> CliResult<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("╭─ FastSecDec ─────────────────────────────────────────────────╮");
        if let Some(object) = value.as_object() {
            for (key, value) in object {
                println!(
                    "  {key:20} {}",
                    if let Some(text) = value.as_str() {
                        text.to_owned()
                    } else {
                        value.to_string()
                    }
                );
            }
        } else {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        println!("╰─────────────────────────────────────────────────────────────╯");
    }
    Ok(())
}

fn integration_report(
    result: &driver::IntegrationReport,
    comparison: Option<&reference::ReferenceReport>,
    json: bool,
) -> CliResult<()> {
    if json {
        let mut value = serde_json::to_value(result)?;
        if let Some(comparison) = comparison {
            value["reference"] = serde_json::to_value(comparison)?;
        }
        return report(&value, true);
    }
    println!("╭─ FastSecDec · Laurent coefficients ───────────────────────────────────╮");
    println!("  Scope: {}", result.scope);
    println!("  {:12} {:>23} {:>15}", "Order", "Value", "Std. error");
    if let Some(estimate) = &result.estimate {
        for (i, order) in estimate.orders.iter().enumerate() {
            println!(
                "  {:12} {:>+23.12e} {:>15.4e}",
                format!("ε^{order} {:?}", estimate.components[i]),
                estimate.mean[i],
                estimate.standard_error[i]
            );
        }
    }
    println!(
        "  {} · {:.2} s",
        result.stopping_reason, result.elapsed_seconds
    );
    if let Some(design) = &result.qmc_design {
        println!("  {design}");
    }
    println!("╰──────────────────────────────────────────────────────────────────────╯");
    if let Some(comparison) = comparison {
        print!("{comparison}");
    }
    Ok(())
}
