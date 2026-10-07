"""The shared short scientific recipe calls only the native caller-owned API."""
from types import SimpleNamespace as NS
from notebook_cells import science


def test_generation_construction_is_inert_eager_and_explicitly_configured():
    calls = []
    session = object()
    class Integral:
        def __init__(self, **arguments): calls.append(("integral", arguments))
        def generation_session(self, **options):
            calls.append(("generation_session", options))
            return session
    namespace = NS(Integral=Integral, CompilationSettings=lambda **kw: kw)
    callbacks = science(namespace)
    assert calls == []
    arguments = {"diagram": object(), "model_parameters": "fixed", "scalar_values": {object(): object()}}
    original = dict(arguments)
    assert callbacks.generation(NS(integral_arguments=lambda: arguments), {"max_order": 2}) is session
    assert arguments == original and calls[0] == ("integral", arguments)
    assert calls[1][1] == {"max_order": 2, "coefficient_expansion": "coefficient_series",
                            "compilation_settings": {"backend": "eager"}}


def test_input_generation_defaults_and_explicit_overrides_reach_the_native_owner():
    calls = []
    defaults = {"mode": "numerical_dual", "subtraction": "taylor",
                "coefficient_expansion": "coefficient_series"}
    native = NS(generation_session=lambda **options: calls.append(options))
    callbacks = science(NS(Integral=lambda **arguments: native,
                           CompilationSettings=lambda **kw: kw))
    prepared = NS(integral_arguments=lambda: {}, generation_arguments=lambda: defaults)
    callbacks.generation(prepared, {"max_order": 0})
    assert calls[-1] == {**defaults, "max_order": 0,
                         "compilation_settings": {"backend": "eager"}}
    override = {"max_order": 1, "mode": "symbolic", "subtraction": "integrate_by_parts",
                "coefficient_expansion": "full_expression"}
    callbacks.generation(prepared, override)
    assert calls[-1] == {**override, "compilation_settings": {"backend": "eager"}}
    assert defaults == {"mode": "numerical_dual", "subtraction": "taylor",
                        "coefficient_expansion": "coefficient_series"}


def test_havana_forwards_all_proposal_options_and_uses_real_batch_adapter():
    received = []
    session = NS(step=lambda **kw: received.append(kw))
    kernels = NS(mc_session=lambda settings, pilot: received.append((settings, pilot)) or session)
    callbacks = science(NS(HavanaDiscreteSettings=lambda **kw: kw))
    config = {"method": "havana_discrete_mc", "pilot_points": 64, "pilot_batches": 2,
              "seed": 7, "bins": 16, "minimum_probability_density": 0.01,
              "maximum_sector_probability_ratio": 100}
    assert callbacks.integration(kernels, config) is session
    settings, pilot = received[0]
    assert pilot and settings == {"points_per_batch": 64, "batches": 2, "seed": 7,
        "bins": 16, "minimum_probability_density": 0.01, "maximum_sector_probability_ratio": 100}
    callbacks.step(session, config["method"])
    assert received[1] == {"max_batches": 1, "evaluation_batch_size": 256}


def test_runtime_point_binding_is_explicit_and_uses_only_declared_native_symbols():
    from symbolica import S
    mt, ymt, mh, gram, unused = S("model::MT", "model::ymt", "model::MH", "gghh_kinematics::dot_0_1", "model::unused", is_real=True)
    calls = []
    bound = object()
    template = NS(runtime_parameters=[mt, ymt, mh, gram],
        with_parameters=lambda values, stability: calls.append((values, stability)) or bound)
    generated = NS(runtime_parameter_defaults={mt: 172.5, ymt: 172.5, mh: 125, unused: 7})
    prepared = NS(runtime_point=lambda point: {gram: point["sqrt_s"]**2 / 2})
    callbacks = science(NS(StabilitySettings=lambda **kw: kw))
    selected, values = callbacks.bind(generated, template, prepared,
        {"top_mass": 170, "higgs_mass": 120, "sqrt_s": 320})
    assert selected is bound and values == {mt: 170.0, ymt: 170.0, mh: 120.0, gram: 51200}
    assert calls == [(values, {"mode": "distance"})]
    assert generated.runtime_parameter_defaults[mt] == 172.5
