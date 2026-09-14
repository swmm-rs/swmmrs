# Hydrologic solver features

Rain has a habit of turning up in the sewer in different ways. Different models
find different ways to parameterize that behavior with benefits and tradeoffs.
The hydrologic features here offer alternative models for modeling runoff and 
I&I. They require explicit selection, and released EPA SWMM does not
recognize them. Existing EPA input stays on the parity-first path.

## Antecedent Moisture Model RDII

The [Antecedent Moisture Model RDII extension](amm-rdii.md) implements
the reparameterized standard and baseflow equations from JWMM C525. Three
explicit input sections select it:

- `[AMM_MODELS]`;
- `[AMM_COMPONENTS]`; and
- `[AMM_RDII]`.

AMM flow uses existing rain gages, temperature sources, RDII interface files,
routing, reporting, checkpoints, and forks. It may be combined with native RTK
RDII at the same node. Released EPA SWMM rejects the AMM sections.

Return to the [solver feature overview](../index.md).
