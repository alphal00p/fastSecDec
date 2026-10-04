mod artifact;
mod config;
mod diagnostics;
mod display;
mod driver;
mod generate;
mod input;

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
    #[arg(long, global = true, help = "Disable the live terminal dashboard")]
    plain: bool,
    #[arg(
        long,
        global = true,
        help = "Stream status snapshots as JSON lines to stderr"
    )]
    status_json: bool,
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Generate portable O2 kernels from a native TOML run card.
    Generate {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate and integrate a native TOML run card.
    Run {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[command(flatten)]
        integration: IntegrationArgs,
    },
    /// Integrate a saved portable artifact, optionally resuming a checkpoint.
    Integrate {
        artifact: PathBuf,
        #[command(flatten)]
        integration: IntegrationArgs,
    },
    /// Inspect native input or an existing portable artifact.
    Inspect {
        path: PathBuf,
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
    /// Probe finite evaluation while approaching every boundary from inside.
    CheckBoundaries {
        artifact: PathBuf,
        #[arg(long, value_delimiter = ',', default_value = "3,6,9,12,15")]
        exponents: Vec<i32>,
    },
}

#[derive(Args, Default)]
struct IntegrationArgs {
    #[arg(long)]
    method: Option<String>,
    #[arg(long)]
    points: Option<u64>,
    #[arg(long)]
    shifts: Option<u32>,
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
}

impl IntegrationArgs {
    fn apply(&self, settings: &mut IntegrationInput) {
        if let Some(value) = &self.method {
            settings.method = value.clone();
        }
        if let Some(value) = self.points {
            settings.points = value;
        }
        if let Some(value) = self.shifts {
            settings.shifts = value;
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
    }
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
    let json = cli.json;
    match run(cli) {
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
            } else if std::io::stderr().is_terminal() {
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
    let make_dashboard = || display::Dashboard::new(!cli.plain && !cli.json, cli.status_json);
    match cli.command {
        Action::Generate { input, output } => {
            let output = output.unwrap_or_else(|| input::artifact_path(&input));
            let mut dashboard = make_dashboard()?;
            let (artifact, kernels) = generate::generate(&input, &output, &mut dashboard)?;
            drop(dashboard);
            report(
                &serde_json::json!({"artifact":output,"content_id":artifact.content_id,"sectors":kernels.sectors().len(),"orders":kernels.orders()}),
                render_json,
            )?;
        }
        Action::Run {
            input,
            output,
            integration,
        } => {
            let output = output.unwrap_or_else(|| input::artifact_path(&input));
            let mut dashboard = make_dashboard()?;
            let (artifact, kernels) = if integration.resume {
                artifact::Artifact::load(&output)?
            } else {
                generate::generate(&input, &output, &mut dashboard)?
            };
            if integration.resume {
                artifact.verify_input_sources(&input)?;
            }
            let mut settings: IntegrationInput =
                serde_json::from_value(artifact.provenance.integration.clone())?;
            integration.apply(&mut settings);
            let checkpoint = integration
                .checkpoint
                .unwrap_or_else(|| output.with_extension("checkpoint.json"));
            let result = driver::integrate(
                &artifact,
                &kernels,
                &settings,
                &checkpoint,
                integration.resume,
                &mut dashboard,
            )?;
            drop(dashboard);
            integration_report(&result, render_json)?;
        }
        Action::Integrate {
            artifact: path,
            integration,
        } => {
            let (artifact, kernels) = artifact::Artifact::load(&path)?;
            let mut settings: IntegrationInput =
                serde_json::from_value(artifact.provenance.integration.clone())?;
            integration.apply(&mut settings);
            let checkpoint = integration
                .checkpoint
                .unwrap_or_else(|| path.with_extension("checkpoint.json"));
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
            integration_report(&result, render_json)?;
        }
        Action::Inspect { path, expressions } => {
            if path
                .extension()
                .is_some_and(|extension| extension == "toml")
            {
                let loaded = input::load(&path)?;
                let mut value = serde_json::json!({"name":loaded.label,"loops":loaded.loops,"parameters":loaded.propagators,
                    "domain":format!("{:?}",loaded.integrand.domain()),"terms":loaded.integrand.terms().len(),
                    "independent_externals":loaded.independent_externals,"dependent_externals":loaded.dependent_externals});
                if expressions {
                    value["density"] =
                        serde_json::Value::String(loaded.integrand.density().to_canonical_string());
                }
                report(&value, render_json)?;
            } else {
                let (artifact, kernels) = artifact::Artifact::load(&path)?;
                report(
                    &serde_json::json!({"content_id":artifact.content_id,"provenance":artifact.provenance,
                    "orders":kernels.orders(),"sectors":kernels.sectors().len(),"dimensions":kernels.sectors().iter().map(|k|k.dimension()).collect::<Vec<_>>(),
                    "exact_coefficients":kernels.exact_coefficients()}),
                    render_json,
                )?;
            }
        }
        Action::Benchmark {
            artifact,
            points,
            repetitions,
        } => {
            let (_, mut kernels) = artifact::Artifact::load(&artifact)?;
            report(
                &serde_json::to_value(diagnostics::benchmark(&mut kernels, points, repetitions)?)?,
                render_json,
            )?;
        }
        Action::CheckBoundaries {
            artifact,
            exponents,
        } => {
            if exponents
                .iter()
                .any(|exponent| !(1..=15).contains(exponent))
            {
                return Err("boundary exponents must lie between 1 and 15 for representable upper-boundary distances".into());
            }
            let (_, mut kernels) = artifact::Artifact::load(&artifact)?;
            let probes = diagnostics::boundaries(&mut kernels, &exponents);
            let failed = probes.iter().filter(|probe| !probe.finite).count();
            report(
                &serde_json::json!({"probes":probes,"failures":failed}),
                render_json,
            )?;
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

fn integration_report(result: &driver::IntegrationReport, json: bool) -> CliResult<()> {
    if json {
        return report(&serde_json::to_value(result)?, true);
    }
    println!("╭─ FastSecDec · Laurent coefficients ───────────────────────────────────╮");
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
    println!("╰──────────────────────────────────────────────────────────────────────╯");
    Ok(())
}
