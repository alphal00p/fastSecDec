//! Bounded scheduling on the CLI-owned pool; the terminal stays on its coordinator.
use std::{
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Worker {
    pub index: usize,
    pub activity: String,
    pub busy: bool,
    pub completed: usize,
    pub active_seconds: f64,
    pub busy_seconds: f64,
    #[serde(skip)]
    started: Option<Instant>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Progress {
    pub completed: usize,
    pub total: usize,
    pub elapsed_seconds: f64,
    pub workers: Vec<Worker>,
}
impl Progress {
    pub fn running(&self) -> usize {
        self.workers.iter().filter(|worker| worker.busy).count()
    }
    pub fn eta_seconds(&self) -> Option<f64> {
        (self.completed > 0).then(|| {
            self.elapsed_seconds * self.total.saturating_sub(self.completed) as f64
                / self.completed as f64
        })
    }
}
enum Message<C> {
    Start(usize, String),
    Activity(usize, String),
    Complete(usize, Result<C, String>),
}
struct CancelOnUnwind<'a>(&'a AtomicBool);
impl Drop for CancelOnUnwind<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

pub(crate) fn run<J: Send, C: Send>(
    pool: &rayon::ThreadPool,
    jobs: &mut dyn ExactSizeIterator<Item = J>,
    cancelled: &Arc<AtomicBool>,
    label: impl Fn(&J) -> String + Sync,
    execute: impl Fn(J, &mut dyn FnMut(String) -> ControlFlow<()>) -> Result<C, String> + Sync,
    mut observe: impl FnMut(&Progress) -> ControlFlow<()>,
) -> Result<Vec<C>, String> {
    let workers = pool.current_num_threads();
    let mut progress = Progress {
        completed: 0,
        total: jobs.len(),
        elapsed_seconds: 0.0,
        workers: (0..workers)
            .map(|index| Worker {
                index,
                activity: "ready".into(),
                busy: false,
                completed: 0,
                active_seconds: 0.0,
                busy_seconds: 0.0,
                started: None,
            })
            .collect(),
    };
    if observe(&progress).is_break() {
        cancelled.store(true, Ordering::Relaxed);
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err("generation cancelled".into());
    }
    let started = Instant::now();
    let cadence = Duration::from_millis(40);
    let (sender, receiver) = mpsc::sync_channel(workers.saturating_mul(4));
    let mut completions = Vec::with_capacity(progress.total);
    let mut failure = None;
    pool.in_place_scope(|scope| {
        let _cancel_on_unwind = CancelOnUnwind(cancelled);
        let receiver = receiver;
        let mut active = 0usize;
        let mut last = Instant::now();
        loop {
            while active < workers && !cancelled.load(Ordering::Relaxed) {
                let Some(job) = jobs.next() else { break };
                let sender = sender.clone();
                let cancelled = Arc::clone(cancelled);
                let label = &label;
                let execute = &execute;
                scope.spawn(move |_| {
                    let index = rayon::current_thread_index().expect("CLI pool worker");
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        let _ = sender.send(Message::Start(index, label(&job)));
                        execute(job, &mut |activity| {
                            if cancelled.load(Ordering::Relaxed) {
                                return ControlFlow::Break(());
                            }
                            let _ = sender.try_send(Message::Activity(index, activity));
                            ControlFlow::Continue(())
                        })
                    }))
                    .unwrap_or_else(|panic| {
                        Err(panic
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).into()))
                            .unwrap_or_else(|| "worker panic".into()))
                    });
                    let _ = sender.send(Message::Complete(index, result));
                });
                active += 1;
            }
            if active == 0 {
                break;
            }
            match receiver.recv_timeout(cadence) {
                Ok(Message::Start(index, activity)) => {
                    let worker = &mut progress.workers[index];
                    worker.busy = true;
                    worker.activity = activity;
                    worker.started = Some(Instant::now());
                }
                Ok(Message::Activity(index, activity)) => {
                    progress.workers[index].activity = activity
                }
                Ok(Message::Complete(index, result)) => {
                    active -= 1;
                    progress.completed += 1;
                    let worker = &mut progress.workers[index];
                    worker.busy = false;
                    worker.completed += 1;
                    worker.busy_seconds += worker
                        .started
                        .take()
                        .map_or(0.0, |start| start.elapsed().as_secs_f64());
                    worker.activity = "idle".into();
                    match result {
                        Ok(value) => completions.push(value),
                        Err(error) => {
                            failure.get_or_insert(error);
                            cancelled.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    failure.get_or_insert_with(|| "worker channel disconnected".into());
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
            }
            if last.elapsed() >= cadence || progress.completed == progress.total {
                progress.elapsed_seconds = started.elapsed().as_secs_f64();
                for worker in &mut progress.workers {
                    worker.active_seconds = worker
                        .started
                        .map_or(0.0, |start| start.elapsed().as_secs_f64());
                }
                if observe(&progress).is_break() {
                    cancelled.store(true, Ordering::Relaxed);
                }
                last = Instant::now();
            }
        }
    });
    if let Some(error) = failure {
        Err(error)
    } else if cancelled.load(Ordering::Relaxed) {
        Err("generation cancelled".into())
    } else {
        Ok(completions)
    }
}
