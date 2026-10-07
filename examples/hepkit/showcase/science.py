"""Short native scientific calls shared by both notebook frontends."""
from symbolica.community.hepkit import sector_decomposition as sd


def generation(prepared, configuration):
    integral = sd.Integral(**prepared.integral_arguments())
    options = {"coefficient_expansion": "coefficient_series"}
    if hasattr(prepared, "generation_arguments"):
        options.update(prepared.generation_arguments())
    for name in ("mode", "subtraction", "coefficient_expansion"):
        if name in configuration:
            options[name] = configuration[name]
    return integral.generation_session(
        max_order=configuration.get("max_order", 0),
        **options,
        compilation_settings=sd.CompilationSettings(backend="eager"),
    )


def integration(kernels, configuration):
    if configuration.get("method") == "havana_discrete_mc":
        return kernels.mc_session(sd.HavanaDiscreteSettings(
            points_per_batch=configuration["pilot_points"],
            batches=configuration["pilot_batches"], seed=configuration["seed"],
            bins=configuration["bins"],
            minimum_probability_density=configuration["minimum_probability_density"],
            maximum_sector_probability_ratio=configuration["maximum_sector_probability_ratio"],
        ), pilot=True)
    return kernels.session(sd.QmcSettings(
        points=configuration["points"], shifts=configuration["shifts"],
        seed=configuration["seed"], package_points=configuration["package_points"],
        rule=configuration.get("rule", "kuo_33002"),
        periodization=configuration.get("periodization", "korobov3"),
    ))


def step(session, method):
    if method == "havana_discrete_mc":
        return session.step(max_batches=1, evaluation_batch_size=256)
    return session.step(max_packages=1, evaluation_batch_size=256)


def bind(generated, kernels, prepared, point):
    """Explicitly select one physical point on independent native evaluators."""
    from symbolica import S
    values = dict(generated.runtime_parameter_defaults)
    if hasattr(prepared, "runtime_point"):
        values.update(prepared.runtime_point(point))
        # These independent Standard Model leaves define this chosen point.
        for name, value in {"MT": point["top_mass"], "ymt": point["top_mass"],
                            "MH": point["higgs_mass"]}.items():
            symbol = S(f"model::{name}")
            if symbol in kernels.runtime_parameters:
                values[symbol] = float(value)
    values = {symbol: values[symbol] for symbol in kernels.runtime_parameters}
    return kernels.with_parameters(values, stability=sd.StabilitySettings(mode="distance")), values
