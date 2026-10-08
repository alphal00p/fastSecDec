# Independent MadLoop reference

This standalone process evaluates the complete one-loop top-quark contribution
to `g g > h h`, including both Higgs-exchange triangles, all six labeled boxes,
and their rational R2 terms. The incoming helicities are `++`. The parameters
and physical momenta match [`../point.json`](../point.json); other quark
Yukawa couplings are zero. This is a matrix element at one point, not a cross
section or an additional massive-bottom contribution.

## Reproduce

The tested installation is `/common/dev/MG5_aMC_v3_7_0`: MadGraph5_aMC@NLO
3.7.0 (2026-01-05), `loop_sm-no_b_mass` model 1.2, CutTools 1.9.3 and OneLOop
3.6. The reference uses the native optimized MadLoop standalone exporter.
Ninja and COLLIER are disabled to keep the required reduction toolchain small.
No physics routine in MadLoop is modified.

From this directory, with Python 3.10 or newer, `six`, GNU Fortran/C++, GNU
make and bash available:

```bash
python3 reproduce.py --mg5-root /common/dev/MG5_aMC_v3_7_0
```

On the test machine the exact command is:

```bash
nix-shell -p 'python311.withPackages (ps: [ ps.six ])' gfortran gnumake --run \
  'export PATH=/nix/store/5km6kqfhjr16939ak9l1crzsh6x96f68-gfortran-wrapper-15.2.0/bin:$PATH; python3 reproduce.py'
```

The existing GNU Fortran 15.2 compiler is selected because the installed
CutTools module files were built with that version. On another machine use
the compiler matching its MadGraph installation, or rebuild its private copy.
The standalone linker also requires its bundled IREGI archive even though the
runtime reduction is CutTools. The script copies that archive from the private
installation, or builds the native target privately if it is absent.
The script copies the installation into ignored `.build/` before running it;
the shared installation is not modified. It generates the process, installs
the cards and small Fortran point driver, rebuilds and evaluates it. `--reuse`
skips process generation while still copying the cards and rebuilding the
driver. Each invocation enforces an overall 10-minute limit and a 15-GiB
process-group RSS limit on Linux, with an additional per-process address-space
limit. A limit failure is reported, not replaced by a result.

`result.json` contains the small validated output and input/source hashes.
Build trees, raw logs and timing/RSS observations remain ignored. The script
checks the model-card parameters against the shared point, the native helicity
row, color projection, polarization components, pole cancellation and agreement
between the native squared matrix element and the extracted complex amplitude.
The final clean process-generation/build/evaluation reproduction completed in
81.94 seconds, with sampled peak process-group RSS 1.16 GiB. Evaluation itself
took 1.04 seconds, including repeated native initialization/stability calls.

## Result and conventions

The recorded physical color coefficient, defined by `M_ab = delta_ab A_++`, is

```text
triangle including R2 = +0.0074539704576528313
boxes including R2    = -0.012697847572064663
A_++                  = -0.0052438771144118299
8 |A_++|^2            =  0.00021998597752841711
```

The total imaginary remainder is `1.43e-33`; the single-pole remainder is
`1.73e-18`, and the double pole is zero. The requested MadLoop relative
stability was `1e-12`. Native return code 329 indicates successful quadruple
precision rescue. Its reported stability of zero means that the compared
answers rounded identically; it is not a claim of exact numerical accuracy.

The color and normalization conversion follows the generated source, without
fitting a phase or multiplicative factor to the FastSecDec result:

- `ML5_0_LoopColorFlowCoefs.dat` has one flow, `Tr(Ta Tb) = delta_ab/2`.
  The native flow is `AMPL(1)+AMPL(2)-sum(AMPL(3:10))`, so `A = JAMPL/2`.
- `CT_interface.f` includes `1/(16*pi^2)` in the loop coefficients already.
- `loop_matrix.f` uses `IDEN=512` and fixed-helicity `HELAVGFACTOR=4`.
  Multiplying its fixed-helicity squared output by 128 removes the initial
  color average and identical-Higgs phase-space factor. This agrees with
  `8*abs(A)^2`; neither result includes an initial-helicity average.
- `helas_calls_ampb_1.f` identifies `AMPL(1)` as the box R2 contribution and
  `AMPL(2)` as the Higgs-exchange triangle R2 contribution. Native loop calls
  put the two triangles in `AMPL(5:6)`, so their color coefficient is
  `(AMPL(2)-AMPL(5)-AMPL(6))/2`; the other six loops belong to the boxes.

The incoming native `VXXXXX` vectors, read from slots 5 through 8 of its
eight-component loop wavefunction, are
`eps1=(0,-1/sqrt(2),-i/sqrt(2),0)` and
`eps2=(0,-1/sqrt(2),+i/sqrt(2),0)` for the momenta in the common point.
They are transverse, null and have `eps1.eps2=-1` in metric `(+---)`.
These are the same incoming polarization conventions as the native HEPKit
example. The driver takes native helicity row 4, `(1,1,0,0)`, directly.

The private MadLoop process contains eight loop amplitudes, two R2 amplitudes
and no UV counterterm amplitudes. The native HEPKit/FastSecDec calculation
retains its dimension-dependent numerator, so its rational terms are included
without separately adding these MadLoop R2 amplitudes.
