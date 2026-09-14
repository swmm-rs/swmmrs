# Governance, trust, and why the solver is private

!!! abstract "Short version, for beings with finite lifespans"
    `swmmrs` tries to reproduce EPA SWMM, not quietly replace it with different
    hydraulics. The solver source is private for now. Prospective contributors
    can get access after a conversation about what they want to work on.

    Do not trust the solver because this page asks nicely. Trust comes from
    reproducible comparisons, documented differences, and claims with visible
    limits. If sustained interest makes public collaboration more useful, the
    solver source will move to Apache-2.0.

## Public position

Software can claim compatibility with almost anything. Physics is less
accommodating. `swmmrs` is an independent implementation of EPA SWMM that puts
fidelity first. It changes the implementation, ownership, packaging, and
interfaces without quietly changing the model underneath them.

EPA SWMM is the behavioral reference. When the two engines disagree, assume
that `swmmrs` is wrong. That assumption changes only when comparison finds an
upstream version difference, a documented EPA issue, or an intentional
departure. Non-EPA formulations must be explicit, opt-in, tested, and
documented. They belong in clearly marked, opt-in branches of the timeline.

A new implementation does not inherit EPA SWMM's authority by borrowing its
name. Compatibility claims must come from measured behavior. `swmmrs` preserves
the model while changing how people build, isolate, and use it. This is not an
attempt to put different hydraulics in a familiar uniform and hope nobody
notices.

## Why the solver source is private

I began `swmmrs` to learn Rust by translating a program I already knew and
cared about. I still build it for myself first. The project is not a service
owed to other developers, and its existence does not promise that the solver
source belongs to the public.

Private source is a social boundary, not a force field. I do not have the
resources to enforce a custom restrictive source license, and I will not
pretend that a sternly worded file can patrol the galaxy. Requiring contact
before access gives interested developers a reason to introduce themselves,
exchange ideas, and contribute. It also discourages silent forks into parallel
timelines with no contact between their inhabitants.

People who genuinely intend to contribute can get access to the solver
repository. Contributions can be code, validation models, discrepancy
investigation, benchmarks, review, documentation, public interfaces, or
tooling. The point is communication and reciprocal work, not exclusivity.

The public repository contains the Python and command-line interfaces, output
reader, documentation, packaging, installers, and release checks. Original
public-repository material is licensed under Apache-2.0. Official wheels and
command-line archives may be used, modified, and redistributed under the
[swmmrs Binary Runtime License](https://github.com/swmm-rs/swmmrs/blob/main/LICENSES/SWMMRS-BINARY-RUNTIME.txt).

Privacy proves correctness about as well as a sealed box with a blinking light.
It also limits independent review of the implementation. That cost is real,
and the project does not ask users to ignore it.

## Trust model

**Private implementation. Public receipts.**

While the source is private, `swmmrs` has to make its case with evidence that
anyone can inspect:

- reproducible [regression comparisons](regression-validation.md) against EPA
  SWMM.
- published benchmark workloads, engine versions, methods, hardware, timings,
  and result distances.
- documented [differences from EPA SWMM](epa-swmm-deviations.md), known upstream
  issues, and solver compatibility changes.
- focused tests and comparison artifacts for intentional behavior changes.
- versioned release artifacts, licenses, and checksums.
- public discussion of reproducible discrepancies and corrections.

These materials make scrutiny possible. They do not prove equivalence across
every model, routing condition, platform, numerical edge case, or neighboring
dimension. Compare critical production results with a trusted EPA SWMM release.
Report unexplained differences with the input model, both engine versions, and
the result artifacts.

There is no Jedi mind trick exemption for numerical software. You do not owe
`swmmrs` trust because its maintainers wrote it. `swmmrs` owes you specific
claims, reproducible evidence, visible limits, and swift corrections.

## Stewarded development while private

While the solver source is private, lead maintainers control repository access,
project scope, and releases. Access begins with a conversation about the work
someone wants to pursue. There is no score, entrance exam, or sentient
admissions computer.

Repository access is not a Jedi rank. Significant changes to numerical methods,
model semantics, defaults, or compatibility expectations need reproducible
evidence, focused validation, and documentation. Another qualified solver
maintainer must review the change when one is available. The project will not
describe a change as independently reviewed when nobody independently reviewed
it.

No captain's chair can make a numerical claim correct. Reference behavior,
reproducible models, benchmarks, published literature, and engineering reasoning
settle technical disagreements. Seniority and repository access do not.

## Request solver access

Useful work can begin on the public side of the airlock:

- report and reduce a behavioral difference from EPA SWMM.
- contribute a difficult validation model or benchmark workload.
- improve comparison methods or investigate a numerical edge case.
- improve public APIs, tooling, packaging, or documentation.
- participate constructively in technical review and design discussion.

A completed public contribution helps, but it is not a formal prerequisite for
access. To start the conversation, email
[admin@swmm.rs](mailto:admin@swmm.rs) with your GitHub username, relevant
experience, and the work you want to pursue.

Do not include private credentials or solver source in public issues, pull
requests, logs, or artifacts.

## Path to Apache-2.0

The solver source is not meant to remain frozen in carbonite on principle.
Privacy is the current collaboration model, not a promise of permanent closure.
If interest in `swmmrs` grows enough, the source will move to Apache-2.0.

The decision will be guided by:

- solver repository access requests.
- GitHub stars.
- broader interest expressed through use, discussion, issues, validation work,
  and contributions.

No ceremonial number of stars opens the vault. Together, these signals show
when public collaboration would help the project more than the current privacy
boundary.
