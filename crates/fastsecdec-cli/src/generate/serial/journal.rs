//! Durable, exclusively owned receipt journal. Never deserialize native work.
use super::jobs::{Job, Request, Response};
use crate::{
    CliResult,
    artifact::{self, Dependency, SourceFingerprint},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
struct Entry {
    request_hash: String,
    response: String,
    response_hash: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct State {
    version: u32,
    build_identity: String,
    input_hash: String,
    #[serde(default)]
    generation_overrides: crate::config::GenerationOverrides,
    dependencies: Vec<Dependency>,
    run_directory: String,
    completed: bool,
    entries: BTreeMap<String, Entry>,
}
pub(super) struct Journal {
    _lock: File,
    directory: PathBuf,
    pub root: PathBuf,
    state: State,
}
impl Journal {
    #[cfg(test)]
    pub fn open(input: &Path, output: &Path, resume: bool, run_id: &str) -> CliResult<Self> {
        Self::open_with_overrides(input, output, resume, run_id, Default::default())
    }

    pub fn open_with_overrides(
        input: &Path,
        output: &Path,
        resume: bool,
        run_id: &str,
        generation_overrides: crate::config::GenerationOverrides,
    ) -> CliResult<Self> {
        let directory = output.with_file_name(format!(
            "{}.generation",
            output
                .file_name()
                .ok_or("missing output basename")?
                .to_string_lossy()
        ));
        fs::create_dir_all(&directory)?;
        if fs::symlink_metadata(&directory)?.file_type().is_symlink() {
            return Err("generation staging directory cannot be a symlink".into());
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("lock"))?;
        lock.try_lock().map_err(|error| {
            format!("another process owns the generation staging directory: {error}")
        })?;
        let input_hash = SourceFingerprint::RunCardScientificInput.hash(&fs::read(input)?)?;
        let dependencies = artifact::dependencies();
        let journal = directory.join("journal.json");
        let previous = if journal.exists() {
            Some(serde_json::from_reader::<_, State>(File::open(&journal)?)?)
        } else {
            None
        };
        let state = if resume {
            let state = previous.ok_or("no resumable generation journal exists")?;
            if state.version != 1
                || state.build_identity != crate::process::child::build_identity()
                || state.input_hash != input_hash
                || state.generation_overrides != generation_overrides
                || state.dependencies != dependencies
            {
                return Err(
                    "generation resume refused: input settings, FastSecDec build or native dependencies changed"
                        .into(),
                );
            }
            state
        } else {
            if previous.as_ref().is_some_and(|old| !old.completed) {
                return Err("unfinished generation staging exists; use --resume or choose a different output basename".into());
            }
            State {
                version: 1,
                build_identity: crate::process::child::build_identity(),
                input_hash,
                generation_overrides,
                dependencies,
                run_directory: format!("run-{run_id}"),
                completed: false,
                entries: BTreeMap::new(),
            }
        };
        if Path::new(&state.run_directory).components().count() != 1
            || !matches!(
                Path::new(&state.run_directory).components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err("invalid generation staging run directory".into());
        }
        let root = directory.join(&state.run_directory);
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join("jobs"))?;
        fs::create_dir_all(root.join("receipts"))?;
        let out = Self {
            _lock: lock,
            directory,
            root,
            state,
        };
        out.save()?;
        Ok(out)
    }
    pub fn completed(&self) -> bool {
        self.state.completed
    }
    /// Children inherit this locked open-file description, so coordinator death
    /// cannot release exclusive residency while native work is still alive.
    pub fn residency_lock(&self) -> &File {
        &self._lock
    }
    pub fn response_path(&self, key: &str) -> PathBuf {
        self.root.join("receipts").join(format!("{key}.json"))
    }
    pub fn job_path(&self, key: &str) -> PathBuf {
        self.root.join("jobs").join(format!("{key}.json"))
    }
    fn save(&self) -> CliResult<()> {
        artifact::atomic_write(
            &self.directory.join("journal.json"),
            &serde_json::to_vec(&self.state)?,
        )?;
        File::open(&self.directory)?.sync_all()?;
        Ok(())
    }
    fn request_hash(request: &Request) -> CliResult<String> {
        let normalized = match request {
            Request::Prepare {
                input, overrides, ..
            } => Request::Prepare {
                input: input.clone(),
                workers: 0,
                overrides: *overrides,
            },
            _ => request.clone(),
        };
        Ok(blake3::hash(&serde_json::to_vec(&normalized)?)
            .to_hex()
            .to_string())
    }
    /// Register immutable request intent before issuing it. An unacknowledged
    /// but durable matching receipt can be adopted after a coordinator crash.
    pub fn request(&mut self, key: &str, request: Request) -> CliResult<(Job, Option<Response>)> {
        if key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err("invalid generation job key".into());
        }
        let request_hash = Self::request_hash(&request)?;
        let response = self.response_path(key);
        if let Some(entry) = self.state.entries.get(key) {
            if entry.request_hash != request_hash {
                return Err(format!("generation resume refused: job {key} changed").into());
            }
        } else {
            self.state.entries.insert(
                key.into(),
                Entry {
                    request_hash,
                    response: format!("receipts/{key}.json"),
                    response_hash: None,
                },
            );
            self.save()?;
        }
        let job = Job {
            root: self.root.clone(),
            response: response.clone(),
            request,
        };
        artifact::atomic_write(&self.job_path(key), &serde_json::to_vec(&job)?)?;
        let existing = if response.exists() {
            Some(self.accept(key, &job.request)?)
        } else {
            if self.state.entries[key].response_hash.is_some() {
                return Err(format!("committed generation receipt {key} is missing").into());
            }
            None
        };
        Ok((job, existing))
    }
    pub fn accept(&mut self, key: &str, request: &Request) -> CliResult<Response> {
        let path = self.response_path(key);
        let bytes = fs::read(&path)?;
        let hash = blake3::hash(&bytes).to_hex().to_string();
        let entry = self
            .state
            .entries
            .get(key)
            .ok_or("unissued generation completion")?;
        if entry.request_hash != Self::request_hash(request)?
            || entry
                .response_hash
                .as_ref()
                .is_some_and(|expected| expected != &hash)
        {
            return Err("generation receipt identity changed".into());
        }
        let response: Response = serde_json::from_slice(&bytes)?;
        validate(request, &response, &self.root)?;
        self.state.entries.get_mut(key).unwrap().response_hash = Some(hash);
        self.save()?;
        Ok(response)
    }
    pub fn complete(&mut self) -> CliResult<()> {
        self.state.completed = true;
        self.save()
    }
    pub fn cleanup(&self) -> CliResult<()> {
        fs::remove_dir_all(&self.root)?;
        Ok(())
    }
}

pub(super) fn validate(request: &Request, response: &Response, root: &Path) -> CliResult<()> {
    match (request, response) {
        (Request::Prepare { input, .. }, Response::Prepared(prepared)) => {
            if prepared.provenance.dependencies != artifact::dependencies() {
                return Err("prepared dependencies changed".into());
            }
            let base = input.parent().unwrap_or_else(|| Path::new("."));
            for source in &prepared.provenance.sources {
                if source
                    .fingerprint
                    .hash(&fs::read(base.join(&source.path))?)?
                    != source.blake3
                {
                    return Err(format!(
                        "generation input {} changed since preparation",
                        source.path
                    )
                    .into());
                }
            }
            prepared.native.source.verify(root)?;
            for map in &prepared.native.charts {
                map.map.verify(root)?;
            }
        }
        (Request::Discover { index, .. }, Response::Discovered(chart)) if *index == chart.index => {
            chart.record.verify(root)?;
        }
        (
            Request::Symmetry {
                chart, candidates, ..
            },
            Response::Symmetry(assignment),
        ) if assignment.source == chart.index => {
            if assignment.representative != chart.index
                && !candidates
                    .iter()
                    .any(|c| c.index == assignment.representative)
            {
                return Err("symmetry completion contains an unissued candidate".into());
            }
            if assignment
                .permutation
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                != (0..chart.dimension).collect()
            {
                return Err("symmetry completion has an invalid permutation".into());
            }
        }
        (Request::Formula { chart, .. }, Response::Formula(formula))
            if chart.formula_key.as_ref() == Some(&formula.key)
                && chart.source_id == formula.source_id =>
        {
            formula.record.verify(root)?;
        }
        (Request::Sector { job, output, .. }, Response::Compiled(compiled))
            if compiled.source_index == job.index && &compiled.data == output =>
        {
            let mut file = File::open(&compiled.data)?;
            let mut buffer = [0u8; 64 * 1024];
            for receipt in &compiled.receipts {
                let mut remaining = receipt.length;
                let mut hash = blake3::Hasher::new();
                while remaining > 0 {
                    let n = remaining.min(buffer.len() as u64) as usize;
                    file.read_exact(&mut buffer[..n])?;
                    hash.update(&buffer[..n]);
                    remaining -= n as u64;
                }
                if hash.finalize().to_hex().as_str() != receipt.digest {
                    return Err("compiled sector digest mismatch".into());
                }
            }
            if file.read(&mut buffer[..1])? != 0 {
                return Err("compiled sector has trailing unclaimed data".into());
            }
        }
        _ => return Err("generation completion does not match its issued job".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn card(path: &Path) {
        fs::write(path,"[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['0']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='1'\nrole='polynomial'\n[generation]\norder=0\n").unwrap();
    }
    #[test]
    fn durable_unacknowledged_prepare_is_adopted_with_a_changed_worker_count() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.toml");
        card(&input);
        let output = dir.path().join("integral.fsd");
        let mut journal = Journal::open(&input, &output, false, "first").unwrap();
        let (_, cached) = journal
            .request(
                "prepare",
                Request::Prepare {
                    input: input.clone(),
                    workers: 1,
                    overrides: Default::default(),
                },
            )
            .unwrap();
        assert!(cached.is_none());
        super::super::jobs::execute(&journal.job_path("prepare"), &mut |_| Ok(())).unwrap();
        let before = fs::read(journal.response_path("prepare")).unwrap();
        drop(journal);
        let mut resumed = Journal::open(&input, &output, true, "second").unwrap();
        let (_, cached) = resumed
            .request(
                "prepare",
                Request::Prepare {
                    input: input.clone(),
                    workers: 3,
                    overrides: Default::default(),
                },
            )
            .unwrap();
        assert!(matches!(cached, Some(Response::Prepared(_))));
        assert_eq!(before, fs::read(resumed.response_path("prepare")).unwrap());
        // A scheduling-only card edit leaves its scientific source fingerprint unchanged.
        fs::write(
            &input,
            fs::read_to_string(&input)
                .unwrap()
                .replace("order=0", "order=0\nserial=true"),
        )
        .unwrap();
        drop(resumed);
        let resumed = Journal::open(&input, &output, true, "third").unwrap();
        drop(resumed);
        fs::write(
            &input,
            fs::read_to_string(&input)
                .unwrap()
                .replace("order=0", "order=1"),
        )
        .unwrap();
        assert!(Journal::open(&input, &output, true, "changed").is_err());
    }
    #[test]
    fn lock_and_receipt_kind_prevent_concurrent_or_foreign_completions() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.toml");
        card(&input);
        let output = dir.path().join("integral.fsd");
        let mut journal = Journal::open(&input, &output, false, "one").unwrap();
        assert!(Journal::open(&input, &output, true, "two").is_err());
        let request = Request::Prepare {
            input,
            workers: 1,
            overrides: Default::default(),
        };
        journal.request("prepare", request.clone()).unwrap();
        let foreign = Response::Symmetry(fastsecdec::generation::streaming::SymmetryAssignment {
            source: 0,
            representative: 0,
            permutation: vec![],
        });
        artifact::atomic_write(
            &journal.response_path("prepare"),
            &serde_json::to_vec(&foreign).unwrap(),
        )
        .unwrap();
        assert!(journal.accept("prepare", &request).is_err());
    }

    #[test]
    fn generation_resume_checks_cli_contour_capability_without_rewriting_input() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.toml");
        card(&input);
        let original = fs::read(&input).unwrap();
        let output = dir.path().join("integral.fsd");
        let overrides = crate::config::GenerationOverrides { contour: true };
        let journal =
            Journal::open_with_overrides(&input, &output, false, "first", overrides).unwrap();
        drop(journal);
        assert!(Journal::open(&input, &output, true, "incompatible").is_err());
        let resumed =
            Journal::open_with_overrides(&input, &output, true, "same", overrides).unwrap();
        drop(resumed);
        assert_eq!(fs::read(&input).unwrap(), original);
    }
}
