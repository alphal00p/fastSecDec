//! Caller-stepped retention for one actual continued physical blow-up.
//!
//! Incomplete stages are retried from their original owners. This is in-memory
//! progress, not durable recovery or a general BM/global-density completion.
use super::super::{Budget, Error};
use super::helpers::Result;
use super::{
    ContinuedSupportedCenter, ContinuedSupportedChart, RecursiveCenterOrigin,
    RelativeRecursiveBlowup,
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum ContinuedChartState {
    Pending { reason: Option<&'static str> },
    Complete(Arc<ContinuedSupportedChart>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContinuedChartAdvance {
    Complete,
    AlreadyComplete,
    Incomplete { reason: &'static str },
}
#[derive(Clone, Debug)]
pub struct ContinuedSupportedFrontier {
    center: Arc<ContinuedSupportedCenter>,
    blowup: Arc<RelativeRecursiveBlowup>,
    namespace: String,
    states: Vec<ContinuedChartState>,
}
#[derive(Clone, Debug)]
pub struct CompletedContinuedCharts {
    center: Arc<ContinuedSupportedCenter>,
    blowup: Arc<RelativeRecursiveBlowup>,
    charts: Vec<Arc<ContinuedSupportedChart>>,
}
impl CompletedContinuedCharts {
    pub fn center(&self) -> &Arc<ContinuedSupportedCenter> {
        &self.center
    }
    pub fn blowup(&self) -> &Arc<RelativeRecursiveBlowup> {
        &self.blowup
    }
    pub fn charts(&self) -> &[Arc<ContinuedSupportedChart>] {
        &self.charts
    }
}
#[derive(Clone, Debug)]
pub enum ContinuedFrontierCompletion {
    Complete(CompletedContinuedCharts),
    Incomplete(ContinuedSupportedFrontier),
}
impl ContinuedSupportedFrontier {
    pub fn new(
        center: Arc<ContinuedSupportedCenter>,
        blowup: Arc<RelativeRecursiveBlowup>,
        namespace: String,
        b: &mut Budget,
    ) -> Result<Self> {
        if !matches!(blowup.cover().center().origin(), RecursiveCenterOrigin::SupportedProblem(c) if Arc::ptr_eq(c,&center))
            || blowup
                .charts()
                .iter()
                .any(|c| !Arc::ptr_eq(c.geometry().center(), blowup.cover().center()))
        {
            return Err(Error::Invalid(
                "continued frontier actual complete blowup owner",
            ));
        }
        b.reserve_slots(blowup.charts().len())?;
        let states = vec![ContinuedChartState::Pending { reason: None }; blowup.charts().len()];
        Ok(Self {
            center,
            blowup,
            namespace,
            states,
        })
    }
    pub fn center(&self) -> &Arc<ContinuedSupportedCenter> {
        &self.center
    }
    pub fn blowup(&self) -> &Arc<RelativeRecursiveBlowup> {
        &self.blowup
    }
    pub fn states(&self) -> &[ContinuedChartState] {
        &self.states
    }
    pub fn complete_inventory(&self) -> bool {
        self.states
            .iter()
            .all(|s| matches!(s, ContinuedChartState::Complete(_)))
    }
    pub fn advance(&mut self, index: usize, b: &mut Budget) -> Result<ContinuedChartAdvance> {
        let state = self
            .states
            .get(index)
            .ok_or(Error::Invalid("continued frontier chart index"))?;
        if matches!(state, ContinuedChartState::Complete(_)) {
            return Ok(ContinuedChartAdvance::AlreadyComplete);
        }
        let chart = self.blowup.charts()[index].clone();
        let proposed = ContinuedSupportedChart::prepare(
            self.center.clone(),
            chart,
            &format!("{}_chart{index}", self.namespace),
            b,
        );
        match proposed {
            Ok(accepted) => {
                self.states[index] = ContinuedChartState::Complete(accepted);
                Ok(ContinuedChartAdvance::Complete)
            }
            Err(Error::ResourceIncomplete(reason)) => {
                self.states[index] = ContinuedChartState::Pending {
                    reason: Some(reason),
                };
                Ok(ContinuedChartAdvance::Incomplete { reason })
            }
            Err(e) => Err(e),
        }
    }
    pub fn try_complete(self) -> ContinuedFrontierCompletion {
        if !self.complete_inventory() {
            return ContinuedFrontierCompletion::Incomplete(self);
        }
        let charts = self
            .states
            .into_iter()
            .map(|s| match s {
                ContinuedChartState::Complete(chart) => chart,
                ContinuedChartState::Pending { .. } => unreachable!("complete inventory checked"),
            })
            .collect();
        ContinuedFrontierCompletion::Complete(CompletedContinuedCharts {
            center: self.center,
            blowup: self.blowup,
            charts,
        })
    }
}
