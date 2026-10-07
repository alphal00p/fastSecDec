"""Native Standard Model gg → HH catalogue and selected-diagram preparation.

Importing this module does no generation, contraction or sampling. The caller
explicitly builds the catalogue, chooses a native diagram, then prepares it.
"""
from dataclasses import dataclass
from pathlib import Path

from symbolica import E, S
from symbolica.community import hepkit as hep
from symbolica.community.tensor import Representation, Tensor, TensorName, dot
from .inputs import ShowcaseInput

# Archived historical inputs remain readable as evidence; this workflow builds
# the current native Standard Model and never uses a stored diagram selector.
ASSETS = Path(__file__).resolve().parents[1] / "fixtures" / "gghh"


def exact(value):
    value = complex(value)
    def real(number):
        numerator, denominator = float(number).as_integer_ratio()
        return E(str(numerator)) / E(str(denominator))
    return real(value.real) + E("1i") * real(value.imag)


def standard_model():
    model = hep.Model.standard_model()
    card = model.default_parameter_card()
    for name, value in {"MT": 172.5, "ymt": 172.5, "MH": 125.0,
                        "WT": 0.0, "WH": 0.0}.items():
        card.set(name, value, 0.0)
    model = model.with_parameter_card(card)
    values = model.scalar_bindings(card)
    return model, values


def process(model):
    """Restrict particles, never individual vertices of the allowed theory."""
    return model.process([21, 21], [25, 25], particle_selection=[6, 21, 25])


@dataclass(frozen=True)
class Catalogue:
    model: object
    scalar_values: dict
    process: object
    result: object

    @property
    def diagrams(self):
        return self.result.diagrams

    @property
    def default_diagram(self):
        return next(diagram for diagram in self.diagrams if diagram.loop_count == 1)

    def selected(self, identity=None):
        if identity is None:
            return self.default_diagram
        return next(diagram for diagram in self.diagrams if diagram.id == identity)


def catalogue(*, progress="auto"):
    model, values = standard_model()
    native_process = process(model)
    result = native_process.generate_diagrams(
        loops=(1, 2), coupling_orders={"QED": 2}, threads=1,
        symmetrize_initial=True, symmetrize_final=True, allow_zero_flow_edges=True,
        maximum_bridges=None, self_energy=None, tadpoles=None, zero_snails=None,
        numerator_grouping=None,
        projector=E("1"), progress=progress,
    )
    if not result.report.completed:
        raise RuntimeError("Native diagram generation did not complete")
    if not any(diagram.loop_count == 1 for diagram in result.diagrams):
        raise RuntimeError("The generated catalogue contains no one-loop diagram")
    return Catalogue(model, values, native_process, result)


@dataclass(frozen=True)
class GGHHInput(ShowcaseInput):
    auxiliary_momenta: tuple
    raw_diagram: object
    raw_numerator: object
    simplified_numerator: object
    gram_symbols: tuple

    def fixed_scalar_values(self):
        # Zero widths are the declared real-mass convention, not numerical
        # integration defaults for freely varying complex propagator masses.
        return {self.model.parameter(name).symbol: E("0") for name in ("WT", "WH")}

    def integral_arguments(self):
        arguments = super().integral_arguments()
        arguments["auxiliary_momenta"] = list(self.auxiliary_momenta)
        arguments["runtime_parameters"] = [symbol for _, _, symbol in self.gram_symbols]
        return arguments

    def runtime_point(self, point):
        """Bind the native physical Gram matrix without regenerating sectors."""
        named, _ = _external_data(self.raw_diagram, point)
        result = {}
        for left, right, symbol in self.gram_symbols:
            value = complex(_dot(named[left][1], named[right][1]))
            if value.imag != 0:
                raise ValueError("The chosen scattering plane requires real Gram values")
            result[symbol] = value.real
        return result

    def generation_arguments(self):
        return {"coefficient_expansion": "coefficient_series"}

    def gram_legend(self):
        """Describe runtime Gram inputs using this diagram's native leg routing."""
        legs = sorted(hep.Amplitude.from_diagram(self.raw_diagram).legs, key=lambda leg: leg.index)
        external = {edge.id: edge.external_index for edge in self.raw_diagram.external_edges}
        by_index = {leg.index: leg for leg in legs}
        basis = self.raw_diagram.loop_momentum_basis
        labels = []
        for index, edge in enumerate(basis.external_edges):
            if edge not in basis.dependent_externals:
                leg = by_index[external[edge]]
                labels.append(f"P({index}): {leg.state} {leg.particle.name}, leg {leg.index}")
        labels += [f"eps{index + 1}: + helicity, incoming gluon leg {leg.index}"
                   for index, leg in enumerate(leg for leg in legs if leg.state == "incoming")]
        return tuple({"Runtime symbol": str(symbol.formatted(show_namespaces=True)),
                      "Left vector": labels[left], "Right vector": labels[right]}
                     for left, right, symbol in self.gram_symbols)


def _tensor(name, components):
    return Tensor.dense(TensorName.vector(name)(Representation.mink(4)), components)


def _dot(left, right):
    product = dot(left, right)
    product.execute()
    return product.result_scalar().expand()


def _external_data(raw, point):
    import math
    if point is None:
        e, mass, cosine = S("gghh_point::energy", "gghh_point::higgs_mass", "gghh_point::cos_theta")
        energy = 150.0  # Polarizations are dimensionless; native fixed-helicity convention.
    else:
        energy = float(point.get("sqrt_s", 300)) / 2
        mass = float(point.get("higgs_mass", 125))
        cosine = float(point.get("cos_theta", 0.8))
        if not all(math.isfinite(x) for x in (energy, mass, cosine)) or mass <= 0 or energy <= mass or abs(cosine) >= 1:
            raise ValueError("Require sqrt(s) > 2 mH > 0 and -1 < cos(theta) < 1")
        e, mass, cosine = exact(energy), exact(mass), exact(cosine)
    legs = sorted(hep.Amplitude.from_diagram(raw).legs, key=lambda leg: leg.index)
    incoming = [leg.index for leg in legs if leg.state == "incoming"]
    outgoing = [leg.index for leg in legs if leg.state == "outgoing"]
    momentum = (e**2 - mass**2) ** E("1/2")
    longitudinal = momentum * cosine
    transverse = momentum * (1 - cosine**2) ** E("1/2")
    zero = E("0")
    vectors = [[e,zero,zero,e], [e,zero,zero,-e],
               [e,transverse,zero,longitudinal], [e,-transverse,zero,-longitudinal]]
    physical = dict(zip(incoming + outgoing, vectors))
    external = {edge.id: edge.external_index for edge in raw.external_edges}
    basis = raw.loop_momentum_basis
    P = hep.Kinematics.external_momentum()
    polarizations = [TensorName.vector(f"gghh::eps{i+1}") for i in range(2)]
    states = [hep.FourMomentum(energy, 0, 0, z).wavefunction("epsilon", hep.Helicity.PLUS)
              for z in (energy, -energy)]
    named = [(P(i), _tensor(f"gghh_data::p{i}", physical[external[edge]]))
             for i, edge in enumerate(basis.external_edges) if edge not in basis.dependent_externals]
    named += [(name.to_expression(), _tensor(f"gghh_data::epsilon{i}", [exact(z) for z in state.components]))
              for i, (name, state) in enumerate(zip(polarizations, states))]
    return named, polarizations


def prepare(*, selected=None, source=None, observer=None):
    """Contract one chosen native owner, preserving all generated graph factors.

    The (+,+), delta_ab projection retains symbolic Gram products; the initial
    integration point is sqrt(s)=300 GeV, mt=172.5 GeV, mH=125 GeV and cos(theta)=4/5.
    It is a single diagram contribution, not a
    gauge-invariant amplitude or a claim of threshold regularization.
    """
    source = source if source is not None else catalogue(progress=observer or "auto")
    raw = source.selected(selected)
    raw.validate()
    legs = sorted(hep.Amplitude.from_diagram(raw).legs, key=lambda leg: leg.index)
    gluons = [leg for leg in legs if leg.particle.pdg_code == 21]
    incoming = [leg.index for leg in legs if leg.state == "incoming"]
    outgoing = [leg.index for leg in legs if leg.state == "outgoing"]
    if len(gluons) != 2 or len(incoming) != 2 or len(outgoing) != 2:
        raise ValueError("Expected native gg → HH external ports")
    K = hep.Kinematics.loop_momentum()
    regulator, dimension = S("gghh::eps", "gghh::D")
    named, polarization_names = _external_data(raw, None)
    auxiliary = tuple(name.to_expression() for name in polarization_names)
    kinematics = hep.Kinematics(dimension,
        momenta=[K(i) for i in range(raw.loop_count)] + [name for name, _ in named])
    gram_symbols = []
    for i, (left, a) in enumerate(named):
        for j, (right, b) in enumerate(named[i:], i):
            value = _dot(a, b)
            # Preserve only native exact structural zeros. Even dimensionless
            # polarization products are runtime inputs: their numerical native
            # wavefunctions must not freeze binary64 normalizations into algebra.
            if value != E("0"):
                value = S(f"gghh_kinematics::dot_{i}_{j}", is_real=True)
                gram_symbols.append((i, j, value))
            kinematics = kinematics.with_scalar_product(left, right, value)
    color = Representation.coad(8).id(*(leg.tensor_index for leg in gluons))
    slots = [[slot for slot in leg.slots if slot.representation == Representation.mink(4)]
             for leg in gluons]
    if any(len(value) != 1 for value in slots):
        raise ValueError("Expected one Lorentz slot on each external gluon")
    polarization = polarization_names[0](slots[0][0]) * polarization_names[1](slots[1][0])
    raw_numerator = raw.numerator_expression(in_lmb=True)
    contracted = (raw_numerator * color * polarization * raw.projector_expression()).with_lorentz_dimension(dimension)
    contracted = contracted.simplify_algebra(
        # Scalar structure alone permits closed tensor networks. Resolve all
        # Lorentz contractions to native scalar products for parametrization.
        contract="dots", color_substitute_cof_dimension_invariants=True,
    ).to_dots()
    if not contracted.is_scalar:
        raise ValueError("Native numerator contraction left free tensor indices")
    simplified = kinematics.apply(contracted).to_expression()
    diagram = hep.sector_decomposition.with_diagram_expressions(
        raw, numerator=simplified, projector=E("1"),
        overall_factor=raw.overall_factor_expression(evaluate=True),
    )
    return GGHHInput(
        f"gg → HH · {raw.name} · {raw.loop_count} loop(s)", source.model, diagram,
        kinematics, regulator, 4 - 2 * regulator, source.scalar_values, 0,
        auxiliary, raw, raw_numerator, simplified, tuple(gram_symbols),
    )
