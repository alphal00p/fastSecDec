"""Explicit fixed points for numerical reference and owner-equivalence controls."""


def fixed_arguments(value):
    arguments = value.integral_arguments()
    arguments.update(model_parameters="fixed", scalar_values=value.scalar_values)
    return arguments
