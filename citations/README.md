# Citing FastSecDec

Use [fastsecdec.bib](fastsecdec.bib) for the software repository and acknowledge
the reference implementation in [pysecdec.bib](pysecdec.bib) and the mathematical
methods in [kaneko-ueda.bib](kaneko-ueda.bib) and
[binoth-heinrich.bib](binoth-heinrich.bib). Record the FastSecDec commit used with
your calculation; the software citation does not claim a journal publication or
an assigned DOI.

The geometry implementation constructs normal cones from polynomial exponent
supports and triangulates them into monomial sector maps. Its mathematical
foundation is [Kaneko–Ueda, sections 3–4](https://arxiv.org/abs/0908.2897).
Endpoint Taylor subtraction and the resulting Laurent coefficients follow the
method described in [Binoth–Heinrich, section II.C](https://arxiv.org/abs/hep-ph/0004013).
These are method acknowledgements, not dependencies on the authors' software.
The [pySecDec paper](https://arxiv.org/abs/1703.09692) acknowledges the reference
implementation that informed FastSecDec's development and scientific checks.

The HEPKit Python bridge adds these native `Citation` objects to
`symbolica.get_citations()` after generation starts or a valid saved kernel
artifact loads. Queries are cumulative, do not reset use, and leave duplicate
merging and display to Symbolica's existing collector. Importing the module,
configuring an input or cancelling before native work adds no FastSecDec entry.
Both marimo notebooks render this collector and offer its BibTeX output.
