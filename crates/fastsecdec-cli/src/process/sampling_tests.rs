//! Actual coordinate traces cross OS process replacement and recovery fences.
//! This fixture never records coordinates in production executables.
use super::{
    ProcessPool, ProcessWorker,
    child::{self, WorkerCommand, WorkerEvent},
};
use fastsecdec::integration::{
    AccuracyTarget, IntegrationProblem, Periodization, QmcSettings, RuleSource, SectorSpec,
    Tolerance,
    mc::HavanaSettings,
    serial::{SerialMethod, SerialReturn, SerialSession, SerialSettings, SerialTask},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

const CONTROL_RUN: &str = "serial-coordinate-process-test";
const CONTROL_LEASE: u64 = 3;

#[derive(Debug, Serialize, Deserialize)]
struct Returned {
    value: SerialReturn,
    coordinates: Vec<u64>,
}
type Worker = ProcessWorker<WorkerEvent<Returned>>;

#[test]
#[ignore = "subprocess fixture"]
fn coordinate_child_fixture() {
    if std::env::var_os("FASTSECDEC_WORKER_CONTROL").is_none() {
        return;
    }
    child::serve(
        CONTROL_RUN.into(),
        CONTROL_LEASE,
        |task: SerialTask, emit: &mut dyn FnMut(Returned) -> std::io::Result<()>| {
            let mut coordinates = Vec::new();
            let value = task
                .evaluate_weighted_batch(3, |points, weights, output| {
                    coordinates.extend(points.iter().map(|value| value.to_bits()));
                    for ((point, weight), row) in points
                        .chunks_exact(2)
                        .zip(weights)
                        .zip(output.chunks_exact_mut(2))
                    {
                        row[0] = point[0] * weight;
                        row[1] = (point[0] * point[0] + point[1]) * weight;
                    }
                    Ok::<_, String>(())
                })
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            emit(Returned { value, coordinates })
        },
    )
    .unwrap();
}

fn event(worker: &mut Worker) -> WorkerEvent<Returned> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(value) = worker.poll().unwrap() {
            return value;
        }
        assert!(
            !worker.output_closed(),
            "coordinate worker closed control unexpectedly"
        );
        assert!(
            worker.try_wait().unwrap().is_none(),
            "coordinate worker exited unexpectedly"
        );
        assert!(
            Instant::now() < deadline,
            "coordinate worker response timed out"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn spawn(pool: &ProcessPool, log: &Path) -> Worker {
    let mut command = Command::new(super::executable().unwrap());
    command.args([
        "--exact",
        "process::sampling_tests::coordinate_child_fixture",
        "--ignored",
        "--nocapture",
    ]);
    let mut worker = pool
        .spawn_native(&mut command, CONTROL_RUN.into(), CONTROL_LEASE, log)
        .unwrap();
    drop(command);
    match event(&mut worker) {
        WorkerEvent::Ready { pid, build } => {
            assert_eq!(pid, worker.pid());
            assert_eq!(build, child::build_identity());
        }
        other => panic!("expected coordinate worker handshake, received {other:?}"),
    }
    worker
}
fn returned(worker: &mut Worker) -> Returned {
    let value = match event(worker) {
        WorkerEvent::Update(value) => value,
        other => panic!("expected coordinate return, received {other:?}"),
    };
    assert!(matches!(event(worker), WorkerEvent::Finished));
    value
}
fn stop(mut worker: Worker) {
    worker.terminate().unwrap();
    assert!(worker.try_wait().unwrap().is_some());
}
fn problem() -> IntegrationProblem {
    IntegrationProblem::new(
        "actual-process-coordinate-control".into(),
        vec![-1, 0],
        vec![SectorSpec {
            id: 0,
            dimension: 2,
        }],
        vec![0., 0.],
    )
    .unwrap()
}
fn settings(mc: bool, double_points: bool) -> SerialSettings {
    SerialSettings {
        method: if mc {
            SerialMethod::Mc(HavanaSettings {
                points_per_batch: 16,
                batches: 3,
                bins: 4,
                seed: 731,
                ..Default::default()
            })
        } else {
            SerialMethod::Qmc(QmcSettings {
                points: 16,
                shifts: 3,
                seed: 731,
                periodization: Periodization::None,
                rule: RuleSource::Supplied(vec![1, 3]),
                ..Default::default()
            })
        },
        double_points,
        max_rounds: Some(2),
        max_in_flight: 2,
        target: AccuracyTarget::LaurentOrder(0),
        tolerance: Tolerance::new(0., 0.).unwrap(),
        pilot_iterations: 1,
        learning_rate: 0.5,
    }
}
fn restore(session: &SerialSession, path: &Path, workers: usize) -> SerialSession {
    fs::write(path, session.checkpoint().unwrap()).unwrap();
    let mut restored = SerialSession::restore(&fs::read(path).unwrap(), &problem()).unwrap();
    restored.set_max_in_flight(workers).unwrap();
    restored
}

#[derive(Default)]
struct Traces {
    first: BTreeSet<Vec<u64>>,
    full: BTreeSet<Vec<u64>>,
}
impl Traces {
    fn fresh(&mut self, value: &Returned) {
        assert!(
            self.first.insert(value.coordinates[..2].to_vec()),
            "distinct real workers repeated a fresh replica's first point"
        );
        assert!(
            self.full.insert(value.coordinates.clone()),
            "distinct real workers repeated a complete sampling sequence"
        );
    }
}

#[test]
fn actual_process_coordinates_remain_unique_across_replacement_refinement_and_resize() {
    for mc in [false, true] {
        for double in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let checkpoint = directory.path().join("session.json");
            let mut traces = Traces::default();
            let mut session = SerialSession::new(problem(), settings(mc, double)).unwrap();
            let pool = ProcessPool::new(2).unwrap();
            let mut a = spawn(&pool, &directory.path().join("a.log"));
            let mut b = spawn(&pool, &directory.path().join("b.log"));
            assert_ne!(a.pid(), b.pid());
            let ta = session.reserve(0).unwrap().unwrap();
            let tb = session.reserve(0).unwrap().unwrap();
            a.send(WorkerCommand::Execute(ta.clone())).unwrap();
            b.send(WorkerCommand::Execute(tb.clone())).unwrap();
            // Accept the later reservation first; keep a complete but not yet
            // admitted receipt from the other process across its replacement.
            let rb = returned(&mut b);
            let ra = returned(&mut a);
            traces.fresh(&ra);
            traces.fresh(&rb);
            session.submit(rb.value).unwrap();
            stop(a);
            stop(b);
            assert_eq!(pool.live(), 0);
            session.release(&ta).unwrap(); // only after its process was reaped
            drop(pool);

            session = restore(&session, &checkpoint, 1);
            let pool = ProcessPool::new(1).unwrap();
            let mut worker = spawn(&pool, &directory.path().join("replacement.log"));
            let retry = session.reserve(0).unwrap().unwrap();
            assert_eq!(retry.identity().stream, ta.identity().stream);
            assert_ne!(retry.identity().run, ta.identity().run);
            assert_ne!(retry.identity().lease, ta.identity().lease);
            worker.send(WorkerCommand::Execute(retry)).unwrap();
            let replayed = returned(&mut worker);
            assert_eq!(
                ra.coordinates, replayed.coordinates,
                "reaped worker's reservation changed on retry"
            );
            assert!(
                session.submit(ra.value).is_err(),
                "old process receipt crossed a recovery fence"
            );
            session.submit(replayed.value).unwrap();
            let third = session.reserve(0).unwrap().unwrap();
            worker.send(WorkerCommand::Execute(third)).unwrap();
            let third = returned(&mut worker);
            traces.fresh(&third);
            session.submit(third.value).unwrap();
            assert!(session.estimate().unwrap().production_complete);
            stop(worker);
            assert_eq!(pool.live(), 0);
            drop(pool);

            session = restore(&session, &checkpoint, 3);
            let pool = ProcessPool::new(3).unwrap();
            let mut workers = (0..3)
                .map(|i| spawn(&pool, &directory.path().join(format!("refinement-{i}.log"))))
                .collect::<Vec<_>>();
            assert_eq!(
                workers
                    .iter()
                    .map(Worker::pid)
                    .collect::<BTreeSet<_>>()
                    .len(),
                3
            );
            let tasks = (0..3)
                .map(|_| session.reserve(0).unwrap().unwrap())
                .collect::<Vec<_>>();
            for (worker, task) in workers.iter_mut().zip(&tasks) {
                assert_eq!(task.point_count(), if double { 32 } else { 16 });
                assert_eq!(task.identity().epoch, if double { 1 } else { 0 });
                worker.send(WorkerCommand::Execute(task.clone())).unwrap();
            }
            for worker in workers.iter_mut().rev() {
                let value = returned(worker);
                traces.fresh(&value);
                session.submit(value.value).unwrap();
            }
            for worker in workers {
                stop(worker);
            }
            assert_eq!(pool.live(), 0);
            assert_eq!(traces.full.len(), 6);
            assert!(session.exhausted());
            assert_eq!(
                session.snapshot().unwrap().sectors[0].replicas,
                if double { 3 } else { 6 }
            );
            assert!(session.estimate().unwrap().production_complete);
        }
    }
}
