//! Pure native projection/statistics checks: no symbolic loading is required.
use fastsecdec::{
    integration::{Periodization, QmcReturn, QmcSession, QmcSettings, RuleSource, SectorSpec},
    results::{ExactContributionPolicy, KernelResultManifest, ResultScope},
    status::CoefficientComponent,
};
use numerica::numerical_integration::qmc::QmcPartial;

fn manifest() -> KernelResultManifest {
    KernelResultManifest {
        kernel_content_id: "scientific-parent".into(),
        orders: vec![0],
        components: vec![CoefficientComponent::Real],
        sectors: [7, 99, 42]
            .into_iter()
            .map(|id| SectorSpec { id, dimension: 1 })
            .collect(),
        exact_coefficients: vec![5.0],
    }
}
fn selected(ids: Vec<u64>, exact_policy: ExactContributionPolicy) -> ResultScope {
    ResultScope::SelectedSectors {
        sector_ids: ids,
        exact_policy,
    }
}

#[test]
fn projection_preserves_native_ids_exact_policy_and_common_shift_covariance() {
    let parent = manifest();
    for (policy, expected) in [
        (ExactContributionPolicy::IncludeAll, 8.0),
        (ExactContributionPolicy::ExcludeAll, 3.0),
    ] {
        let scope = selected(vec![99, 7], policy);
        assert_eq!(
            parent.canonical_scope(&scope).unwrap(),
            selected(vec![7, 99], policy)
        );
        let problem = parent
            .integration_problem(&scope, "outer-artifact")
            .unwrap();
        assert_eq!(problem.content_id, "outer-artifact");
        assert_eq!(
            problem.sectors.iter().map(|s| s.id).collect::<Vec<_>>(),
            [7, 99]
        );
        let mut session = QmcSession::democratic(
            problem,
            QmcSettings {
                points: 8,
                shifts: 4,
                package_points: 8,
                periodization: Periodization::None,
                rule: RuleSource::Supplied(vec![1]),
                ..Default::default()
            },
        )
        .unwrap();
        while let Some(task) = session.next_work().unwrap() {
            let worker = session.worker_context(task.sector_id()).unwrap();
            let mut partial = QmcPartial::new(worker.plan(), task.work(), 1).unwrap();
            for index in task.work().start()..task.work().start() + task.point_count() {
                let shift = (index / 8) as f64;
                partial
                    .push(&[if task.sector_id() == 7 {
                        shift
                    } else {
                        3.0 - shift
                    }])
                    .unwrap();
            }
            session
                .submit(QmcReturn::from_partial(task, partial, 0.0).unwrap())
                .unwrap();
        }
        let contributions = session.contributions().unwrap();
        assert_eq!(contributions.total.as_ref().unwrap().mean, [expected]);
        assert_eq!(contributions.total.as_ref().unwrap().standard_error, [0.0]);
        assert!(
            contributions
                .sectors
                .iter()
                .all(|row| row.estimate.as_ref().unwrap().standard_error[0] > 0.0)
        );
        assert_eq!(contributions.sectors.len(), 2);
    }
}

#[test]
fn explicit_empty_and_all_selections_never_become_full_and_invalid_ids_fail() {
    let parent = manifest();
    for policy in [
        ExactContributionPolicy::IncludeAll,
        ExactContributionPolicy::ExcludeAll,
    ] {
        let empty = selected(vec![], policy);
        assert_eq!(parent.canonical_scope(&empty).unwrap(), empty);
        let problem = parent.integration_problem(&empty, "outer").unwrap();
        assert!(problem.sectors.is_empty());
        assert_eq!(
            problem.exact_coefficients,
            [if policy == ExactContributionPolicy::IncludeAll {
                5.0
            } else {
                0.0
            }]
        );
        assert!(
            !parent
                .canonical_scope(&selected(vec![99, 42, 7], policy))
                .unwrap()
                .is_full_integral()
        );
        assert!(
            parent
                .integration_problem(&selected(vec![7, 7], policy), "outer")
                .is_err()
        );
        assert!(
            parent
                .integration_problem(&selected(vec![8], policy), "outer")
                .is_err()
        );
    }
    assert_eq!(
        parent.canonical_scope(&ResultScope::default()).unwrap(),
        ResultScope::FullIntegral
    );
}
