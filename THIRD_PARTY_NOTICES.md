# Third-party notices

`swmmrs` builds on software created by other projects. The notices below apply
only to the identified upstream material and do not change the licensing of
original `swmmrs` code.

## EPA SWMM and Open Water Analytics SWMM

The native solver is a Rust translation of EPA SWMM that also incorporates
material from the Open Water Analytics SWMM project.

- Material original to USEPA SWMM v5.1.14 is identified by the upstream project
  as public-domain United States Government work.
- Material created by Open Water Analytics contributors is available under the
  MIT License unless otherwise noted.

The upstream license and contributor list are reproduced in:

- [`thirdparty/licenses/OWA-SWMM-MIT.txt`](thirdparty/licenses/OWA-SWMM-MIT.txt)
- [`thirdparty/licenses/OWA-SWMM-CONTRIBUTORS.txt`](thirdparty/licenses/OWA-SWMM-CONTRIBUTORS.txt)

Upstream project: <https://github.com/OpenWaterAnalytics/Stormwater-Management-Model>

## swmmrs-parallel

The solver uses `swmmrs-parallel`, which is distributed under the Apache
License 2.0. Its license is reproduced in
[`thirdparty/licenses/SWMMRS-PARALLEL-APACHE-2.0.txt`](thirdparty/licenses/SWMMRS-PARALLEL-APACHE-2.0.txt).

## SWMMEnablement model fixtures

Selected regression inputs were copied or adapted from
[SWMMEnablement/1729-SWMM5-Models](https://github.com/SWMMEnablement/1729-SWMM5-Models)
at revision `f2965816381a474f9b1ba7483408a0390a44278e`. That repository is
released under the Unlicense. The source-to-fixture mappings and modifications
are recorded in
[`crates/run/tests/data/regression-suite/CITATION.txt`](crates/run/tests/data/regression-suite/CITATION.txt),
and the license is reproduced in
[`thirdparty/licenses/SWMM5-MODELS-UNLICENSE.txt`](thirdparty/licenses/SWMM5-MODELS-UNLICENSE.txt).

## pyswmm numerical-regression fixtures

Selected regression inputs were copied or adapted from
[pyswmm/swmm-nrtestsuite](https://github.com/pyswmm/swmm-nrtestsuite). The
upstream notice identifies United States Government material as public domain
in the United States and licenses the remaining material under Creative Commons
Attribution 4.0 International. Renamed and modified fixtures are identified in
[`crates/run/tests/data/regression-suite/CITATION.txt`](crates/run/tests/data/regression-suite/CITATION.txt).
The upstream copyright and license notice is reproduced in
[`thirdparty/licenses/PYSWMM-NRTESTSUITE-CC-BY-4.0.txt`](thirdparty/licenses/PYSWMM-NRTESTSUITE-CC-BY-4.0.txt),
and its referenced contributor attribution is reproduced in
[`thirdparty/licenses/PYSWMM-NRTESTSUITE-CONTRIBUTORS.txt`](thirdparty/licenses/PYSWMM-NRTESTSUITE-CONTRIBUTORS.txt).

## swmm-pandas binary-output fixture

`crates/output/tests/fixtures/legacy/Model.out` was copied without regeneration
from [karosc/swmm-pandas](https://github.com/karosc/swmm-pandas) at revision
`410f19013149f6c995251d7df888803b1c70fdd8`. The artifact is used only as an
immutable compatibility fixture. Its Creative Commons Attribution 4.0
International license is reproduced in
[`thirdparty/licenses/SWMM-PANDAS-CC-BY-4.0.txt`](thirdparty/licenses/SWMM-PANDAS-CC-BY-4.0.txt),
and exact provenance is recorded in
[`crates/output/tests/fixtures/PROVENANCE.toml`](crates/output/tests/fixtures/PROVENANCE.toml).

These references provide attribution only and do not imply endorsement by EPA,
Open Water Analytics, pyswmm, SWMMEnablement, or any upstream contributor.
