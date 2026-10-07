"""Caller-owned phase steering, with no independent sampler or estimator."""
from pathlib import Path
import sys
from types import SimpleNamespace as NS

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.state import RunState
from notebook_cells import science
from showcase.integration import coverage_summary, sector_rows


MC = {"method": "havana_discrete_mc", "pilot_points": 256, "pilot_batches": 2,
      "points_per_batch": 1024, "batches": 4, "bins": 16, "seed": 7,
      "minimum_probability_density": 0.01, "maximum_sector_probability_ratio": 100}


def state_backend():
    calls = {"create": 0, "steps": 0, "adapt": 0, "freeze": 0, "checkpoint": 0, "restore": 0}
    class Session:
        stage = "pilot"
        complete = False
        accepted = 0
        @property
        def checkpoint_available(self): return self.stage == "production"
        def snapshot(self): return NS(estimate=None, completed_points=self.accepted, stage=self.stage)
        def step(self, max_batches, evaluation_batch_size):
            assert max_batches == 1 and evaluation_batch_size == 256
            calls["steps"] += 1
            self.accepted += 1
            self.complete = self.accepted == 2
            return self.snapshot()
        def checkpoint(self):
            assert self.stage == "production", "Pilot must stay in the same native owner"
            calls["checkpoint"] += 1
            return str(self.accepted).encode()
        def adapt_pilot(self):
            assert self.complete and self.stage == "pilot"
            calls["adapt"] += 1
            self.accepted = 0
            self.complete = False
            return self.snapshot()
        def freeze_production(self, **settings):
            assert self.complete and self.stage == "pilot"
            assert settings == {"points_per_batch": 1024, "batches": 4}
            calls["freeze"] += 1
            self.stage = "production"
            self.accepted = 0
            self.complete = False
            return self.snapshot()
    class Kernels:
        def mc_session(self, settings, pilot):
            assert pilot is True and settings["points_per_batch"] == 256
            assert "periodization" not in settings
            calls["create"] += 1
            return Session()
        def restore_mc(self, checkpoint):
            calls["restore"] += 1
            value = Session()
            value.stage = "production"
            value.accepted = int(checkpoint)
            return value
    fs = NS(HavanaDiscreteSettings=lambda **kw: kw)
    state = RunState(kernels=Kernels(), configuration={"example": "triangle", "mass": 1,
                    "max_order": 0, "points": 1024, "rule": "kuo_33002", "periodization": "korobov3"})
    return state, fs, calls


def test_pilot_pause_keeps_same_session_and_never_requests_a_checkpoint():
    state, fs, calls = state_backend()
    state.integrate(science(fs).create_session, MC)
    owner = state.session
    assert state.phase == "pilot" and calls["steps"] == 0
    assert "rule" not in state.configuration and "periodization" not in state.configuration
    state.advance(science(fs).advance_session)
    state.cancel()
    assert state.session is owner and state.checkpoint_bytes is None
    assert state.phase == "paused" and not state.active
    state.advance(science(fs).advance_session)
    assert calls["steps"] == 1
    state.resume()
    assert state.session is owner and state.active
    state.advance(science(fs).advance_session)
    assert state.phase == "pilot_ready" and not state.active
    assert calls["checkpoint"] == calls["restore"] == 0
    assert calls["create"] == 1


def test_adapt_and_freeze_are_explicit_and_production_restores_checkpoint():
    state, fs, calls = state_backend()
    state.integrate(science(fs).create_session, MC)
    state.pilot_action(freeze=True)
    assert calls["freeze"] == 0
    state.advance(science(fs).advance_session); state.advance(science(fs).advance_session)
    state.advance(science(fs).advance_session)
    assert calls["steps"] == 2 and calls["freeze"] == 0
    state.pilot_action()
    assert state.phase == "pilot" and calls["adapt"] == 1
    state.advance(science(fs).advance_session); state.advance(science(fs).advance_session)
    state.history.append({"pilot": "must not survive into production"})
    owner = state.session
    state.pilot_action(freeze=True)
    assert state.session is owner and state.history == []
    assert state.phase == "integrating" and calls["steps"] == 4
    state.advance(science(fs).advance_session)
    state.cancel()
    assert state.checkpoint_bytes == b"1"
    state.resume()
    assert state.session is not owner and state.session.accepted == 1
    state.advance(science(fs).advance_session)
    assert state.phase == "complete" and state.checkpoint_bytes == b"2"
    assert calls["restore"] == 1


def test_pilot_keyboard_interrupt_retains_owner_and_accepted_prefix():
    state, fs, calls = state_backend()
    state.integrate(science(fs).create_session, MC)
    state.advance(science(fs).advance_session)
    owner = state.session
    state.fail(KeyboardInterrupt())
    assert state.error is None and state.phase == "interrupted"
    assert state.session is owner and owner.accepted == 1
    state.resume()
    assert state.session is owner and state.active
    assert calls["checkpoint"] == 0


def test_mc_coverage_uses_global_batches_and_native_allocation_probabilities():
    snapshot = NS(method="havana_discrete_mc", sectors=[
        NS(id=i, dimension=2, completed_points=count, planned_points=None,
           complete_replicas=3, planned_replicas=8, worker_seconds=0.1,
           discrete_allocation=NS(probability=probability, points_per_batch=100))
        for i, count, probability in [(0, 280, 0.9), (1, 20, 0.1)]])
    assert coverage_summary(snapshot) == ("Complete global batches", 3, 8)
    rows = sector_rows(snapshot)
    assert [row["accepted points"] for row in rows] == [280, 20]
    assert [row["native selection probability"] for row in rows] == [0.9, 0.1]
    assert all(row["planned points"] == "as allocated" for row in rows)
    assert all("complete shifts" not in row for row in rows)
