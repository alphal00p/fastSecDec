//! CLI-owned bounded scheduling of native opaque chart/cone work.
use fastsecdec::generation::{
    DecompositionProgress, GeometryCompletion, GeometryJob, GeometryJobId, SectorError,
};
use std::{
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, SyncSender},
    },
    time::{Duration, Instant},
};

pub(super) struct Progress {
    pub completed: usize,
    pub total: usize,
    pub running: usize,
    pub latest: Option<(GeometryJobId, DecompositionProgress)>,
}

enum Message {
    Progress(GeometryJobId, DecompositionProgress),
    Complete(Result<GeometryCompletion, String>),
}

struct CancelOnUnwind<'a>(&'a AtomicBool);

impl Drop for CancelOnUnwind<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

fn send_completion(sender: &SyncSender<Message>, execute: impl FnOnce() -> GeometryCompletion) {
    let result = catch_unwind(AssertUnwindSafe(execute)).map_err(|panic| {
        panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
            .unwrap_or_else(|| "non-string panic payload".into())
    });
    let _ = sender.send(Message::Complete(result));
}

fn spawn<'scope>(
    scope: &rayon::Scope<'scope>,
    job: GeometryJob,
    sender: SyncSender<Message>,
    cancelled: Arc<AtomicBool>,
) {
    scope.spawn(move |_| {
        send_completion(&sender, || {
            job.run(|progress| {
                if cancelled.load(Ordering::Relaxed) {
                    return ControlFlow::Break(());
                }
                // Dropping an observation cannot discard the opaque completion.
                let _ = sender.try_send(Message::Progress(job.id(), progress.clone()));
                ControlFlow::Continue(())
            })
        });
    });
}

pub(super) fn run(
    pool: &rayon::ThreadPool,
    workers: usize,
    jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>,
    cancelled: &Arc<AtomicBool>,
    mut observe: impl FnMut(&Progress) -> ControlFlow<()>,
) -> Result<Vec<GeometryCompletion>, SectorError> {
    assert!(workers > 0);
    let mut progress = Progress {
        completed: 0,
        total: jobs.len(),
        running: 0,
        latest: None,
    };
    if observe(&progress).is_break() {
        cancelled.store(true, Ordering::Relaxed);
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(SectorError::Cancelled);
    }
    let (sender, receiver) = mpsc::sync_channel(workers.saturating_mul(2));
    let mut completions = Vec::new();
    let mut failure = None;
    let cadence = Duration::from_millis(40);
    let mut last = Instant::now();
    // This closure stays on the terminal's coordinator thread. Scope exit joins
    // every launched worker, including after cancellation or worker failure.
    pool.in_place_scope(|scope| {
        let _cancel_on_unwind = CancelOnUnwind(cancelled);
        // Drop inside the scope body if the coordinator panics, before Rayon's
        // join. Otherwise a bounded completion send could wait on a dead reader.
        let receiver = receiver;
        let mut active = 0;
        let mut launched = false;
        loop {
            while active < workers && !cancelled.load(Ordering::Relaxed) {
                let Some(job) = jobs.next() else { break };
                spawn(scope, job, sender.clone(), Arc::clone(cancelled));
                active += 1;
            }
            progress.running = active;
            if !launched && active > 0 {
                launched = true;
                if observe(&progress).is_break() {
                    cancelled.store(true, Ordering::Relaxed);
                }
            }
            if active == 0 {
                break;
            }
            match receiver.recv_timeout(cadence) {
                Ok(Message::Progress(id, status)) => progress.latest = Some((id, status)),
                Ok(Message::Complete(result)) => {
                    active -= 1;
                    progress.running = active;
                    progress.completed += 1;
                    match result {
                        Ok(completion) => completions.push(completion),
                        Err(error) => {
                            failure.get_or_insert(error);
                            cancelled.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    failure.get_or_insert_with(|| "geometry result channel disconnected".into());
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
            }
            if last.elapsed() >= cadence || progress.completed == progress.total {
                if observe(&progress).is_break() {
                    cancelled.store(true, Ordering::Relaxed);
                }
                last = Instant::now();
            }
        }
    });
    if let Some(error) = failure {
        return Err(SectorError::Geometry(format!(
            "CLI geometry worker failed: {error}"
        )));
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(SectorError::Cancelled);
    }
    // Native admission checks full coverage, provenance and canonical errors.
    Ok(completions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_panic_is_transported_and_empty_stage_needs_no_worker() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        pool.in_place_scope(|scope| {
            scope.spawn(|_| send_completion(&sender, || panic!("worker control")));
            assert!(matches!(receiver.recv_timeout(Duration::from_secs(2)).unwrap(), Message::Complete(Err(reason)) if reason == "worker control"));
        });
        let cancelled = Arc::new(AtomicBool::new(false));
        let mut observations = 0;
        let complete = run(&pool, 2, &mut std::iter::empty(), &cancelled, |p| {
            assert_eq!((p.completed, p.total, p.running), (0, 0, 0));
            observations += 1;
            ControlFlow::Continue(())
        })
        .unwrap();
        assert!(complete.is_empty());
        assert_eq!(observations, 1);
    }

    #[test]
    fn active_geometry_cancellation_and_coordinator_unwind_join_workers() {
        use fastsecdec::generation::{GenerationContext, GenerationOptions};
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.toml");
        std::fs::write(
            &input,
            r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
[[direct.terms.factors]]
polynomial="1+x+y"
exponent="-1"
"#,
        )
        .unwrap();
        let loaded = crate::input::load(&input).unwrap();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        for panic in [false, true] {
            let cancelled = Arc::new(AtomicBool::new(false));
            let mut context = GenerationContext::new(1);
            let mut saw_running = false;
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
                    run(&pool, 2, jobs, &cancelled, |p| {
                        if p.running > 0 {
                            saw_running = true;
                            assert!(!panic, "coordinator control");
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    })
                };
                context.generate_with_dispatch(
                    &loaded.integrand,
                    &GenerationOptions::default(),
                    &mut dispatch,
                    || cancelled.load(Ordering::Relaxed),
                    |_| ControlFlow::Continue(()),
                )
            }));
            assert!(saw_running && cancelled.load(Ordering::Relaxed));
            assert!(context.geometry_cache().is_empty());
            if panic {
                assert!(outcome.is_err())
            } else {
                assert!(outcome.unwrap().is_err())
            }
        }
    }
}
