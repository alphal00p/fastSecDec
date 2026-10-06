"""Exercise the notebook's actual public-state cells without numerical work."""
from types import SimpleNamespace as NS

from test_state import CONFIG, backend
from test_mc_state import MC, state_backend
from showcase.state import RunState


from notebook_cells import cell_defining, science


class Notebook:
    """Only UI plumbing is doubled; execute actual notebook dispatch and RunState."""
    def __init__(self):
        self.updates = []
        self.refresh_created = 0
        self.active = False
        def state(initial):
            self.active = initial
            def setter(value):
                self.updates.append(value)
                self.active = value
            return lambda: self.active, setter
        def refresh(**kwargs):
            self.refresh_created += 1
            return NS(value="", options=kwargs)
        null = lambda *args, **kwargs: None
        self.mo = NS(state=state, ui=NS(button=lambda **kw: NS(value=kw["value"]), refresh=refresh),
                     callout=null, md=null, vstack=null, Html=null, as_html=null, output=NS(replace=null))
        creator, tree = cell_defining("run_state")
        values = creator(self.mo, NS(RunState=RunState))
        names = [item.id for item in tree.body[-1].value.elts]
        self.env = dict(zip(names, values))
        self.clock, _ = cell_defining("refresh")
        self.action, _ = cell_defining("run_revision")
        fs, prepared, self.calls = backend()
        self.env.update(mo=self.mo, sd=fs, **vars(science(fs)), draft={**CONFIG, "method": "qmc"},
                        allocation_controls=NS(value={}), builders=NS(prepare=lambda _: prepared),
                        gghh_builder=None, presentation=NS(validate_configuration=lambda _: None),
                        generation=NS(generation_view=null),
                        integration=NS(result_view=null, previous_result_view=null))
        self.update_clock()

    @property
    def run(self): return self.env["run_state"]

    def update_clock(self):
        self.env["refresh"], = self.clock(self.env["get_sampling_active"], self.mo)

    def dispatch(self, action=None):
        names = {"new": "new_integration_button"}
        if action == "tick":
            self.env["refresh"].value += "t"
        elif action is not None:
            self.env[names.get(action, action + "_button")].value += 1
        prior = len(self.updates)
        self.action(**{name: self.env[name] for name in self.action.__code__.co_varnames[:self.action.__code__.co_argcount]})
        if len(self.updates) != prior:
            self.update_clock()


def test_timer_absent_until_integrate_and_stable_across_steps():
    notebook = Notebook()
    owner = notebook.run
    buttons = {key: value for key, value in notebook.env.items() if key.endswith("_button")}
    notebook.dispatch()
    assert notebook.env["refresh"] is None and notebook.calls["generate"] == 0
    notebook.dispatch("generate")
    assert notebook.env["refresh"] is None and notebook.calls["step"] == 0
    notebook.dispatch("integrate")
    clock = notebook.env["refresh"]
    assert clock is not None and notebook.updates == [True]
    notebook.dispatch("tick"); notebook.dispatch("tick")
    assert notebook.calls["step"] == 2 and notebook.env["refresh"] is clock
    assert notebook.run is owner and all(notebook.env[key] is value for key, value in buttons.items())
    assert notebook.refresh_created == 1 and notebook.updates == [True]


def test_cancel_stale_tick_resume_completion_and_error_disarm():
    notebook = Notebook()
    notebook.dispatch("generate"); notebook.dispatch("integrate"); notebook.dispatch("tick")
    stale = notebook.env["refresh"]
    notebook.dispatch("cancel")
    assert notebook.env["refresh"] is None and notebook.updates == [True, False]
    # An already queued browser event cannot restart a paused native session.
    notebook.env["refresh"] = stale
    notebook.dispatch("tick")
    assert notebook.calls["step"] == 1 and not notebook.run.active
    notebook.update_clock()
    notebook.dispatch("resume")
    assert notebook.run.active and notebook.env["refresh"] is not None
    notebook.run.session.complete = True
    notebook.dispatch("tick")
    assert not notebook.run.active and notebook.env["refresh"] is None
    assert notebook.updates == [True, False, True, False]

    broken = Notebook()
    broken.dispatch("generate"); broken.dispatch("integrate")
    def failure(**kwargs): raise ValueError("numerical failure")
    broken.run.session.step = failure
    broken.dispatch("tick")
    assert broken.run.error and broken.env["refresh"] is None and broken.updates == [True, False]


def test_pilot_completion_adapt_and_freeze_remain_explicit():
    notebook = Notebook()
    run, fs, calls = state_backend()
    notebook.env.update(run_state=run, sd=fs, **vars(science(fs)), draft={"example": "triangle", "method": MC["method"]},
                        allocation_controls=NS(value={key: value for key, value in MC.items() if key != "method"}))
    notebook.dispatch("integrate")
    owner = run.session
    notebook.dispatch("tick"); notebook.dispatch("cancel"); notebook.dispatch("resume")
    assert run.session is owner and calls["restore"] == 0
    notebook.dispatch("tick")
    assert run.phase == "pilot_ready" and notebook.env["refresh"] is None
    notebook.dispatch()  # No timer or implicit pilot-to-production transition.
    assert calls["freeze"] == calls["adapt"] == 0
    notebook.dispatch("adapt")
    notebook.dispatch("tick"); notebook.dispatch("tick")
    assert calls["adapt"] == 1 and notebook.env["refresh"] is None
    notebook.dispatch("freeze")
    assert run.session is owner and run.active and notebook.env["refresh"] is not None
    notebook.dispatch("tick"); notebook.dispatch("tick")
    assert run.phase == "complete" and notebook.env["refresh"] is None
    assert notebook.updates == [True, False, True, False, True, False, True, False]
