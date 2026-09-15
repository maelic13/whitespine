# Whitespine experiment ledger

This file records every measurement that decides something: game tests
(SPRTs, gauntlets, tournaments), tuning runs (SPSA), evaluation fits (Texel),
data generation, and observations such as speed baselines. `PLAN.md` says what
to do and in what order; this file says what was measured and what it meant.

The maintainer writes every entry. The worked examples in section 3 are
illustrations of the format with invented placeholders, **not results**.

## Contents

- [1. How to use this ledger](#1-how-to-use-this-ledger)
- [2. Templates](#2-templates)
- [3. Worked examples (not real results)](#3-worked-examples-not-real-results)
- [4. Ledger](#4-ledger)
  - [Phase 0 — Foundations and baseline](#phase-0--foundations-and-baseline)
  - [Phase 1 — Board](#phase-1--board)
  - [Phase 2 — Engine rewrite](#phase-2--engine-rewrite)
  - [Phase 3 — Search fundamentals](#phase-3--search-fundamentals)
  - [Phase 4 — Classical evaluation](#phase-4--classical-evaluation)
  - [Phase 5 — Evaluation tuning](#phase-5--evaluation-tuning)
  - [Phase 6 — Advanced search](#phase-6--advanced-search)
  - [Phase 7 — Clock, threads, robustness](#phase-7--clock-threads-robustness)
  - [Phase 8 — Performance and classical checkpoint](#phase-8--performance-and-classical-checkpoint)
  - [Phase 9 — NNUE](#phase-9--nnue)
- [5. Calibration notes](#5-calibration-notes)

## 1. How to use this ledger

**IDs.** `WS-001`, `WS-002`, … in the order entries are *registered*, never
reused. Put the ID in the commit message of the change it tests, in the lab
results folder name (`D:/chess/whitespine-lab/results/WS-017/`) and in the
PLAN step when a result changes a later decision.

**Register before, record after.** Fill in the registration part before the
first game, fit epoch or tuning iteration. The prediction is then frozen: do
not edit it after seeing any result. Add the result, the disposition and the
calibration below it. Correct only a clerical error, and mark the correction.

**Kinds of entry.**

| Kind | Decides | Minimum content |
|---|---|---|
| SPRT | Accept or reject a change | Full SPRT template |
| Gauntlet / tournament | Standing against other engines | Engines with hashes, games per pair, anchor, ratings with error |
| SPSA | Tuned values | Parameter list with ranges and steps, iterations, games per iteration, final values, follow-up SPRT ID |
| Texel fit | A parameter vector | Corpus and manifest hash, K, optimiser settings, losses, vector hash, follow-up SPRT ID |
| Data generation | A corpus | Generator commit and binary hash, settings, counts, file hashes |
| Observation | Nothing by itself | What was measured, how, and on which binary |

**Dispositions.** *Accepted* (passed its gate and entered the head);
*rejected* (failed its gate, change reverted); *neutral* (no distinguishable
effect at the tested resolution); *observation* (a measurement, not a verdict);
*abandoned* (stopped for a stated reason, such as a broken binary or a busy
host; the games are not evidence).

**Evidence layers.** Perft counts, bench node counts, NPS, tactical-suite
scores and fit loss are not Elo. Only games decide strength.

## 2. Templates

### SPRT

```markdown
### WS-NNN — <short name>

- Date registered:
- PLAN step:
- Baseline: <version or tag> / <commit> / <binary SHA-256> / bench <nodes>
- Candidate: <version> / <commit> / <binary SHA-256> / bench <nodes>
- Change and hypothesis: <what changed and why it should gain>
- Bounds: [elo0, elo1] normalized, alpha 0.05, beta 0.05
- Conditions: TC <3+0.03>, Threads 1, Hash 16, book <file>, concurrency 14,
  -use-affinity, adjudication off, max rounds <n>
- Command: <the exact fastchess command, or the path of a file holding it>
- **Prediction (frozen before the first game):**
  - Expected result: <accept / reject / unsure> with <probability>
  - Expected size: <range in Elo, if you dare>
  - Most likely way to be wrong: <...>
- Result: games <n>, W-D-L <w-d-l>, pentanomial <...>, Elo <x ± y>,
  nElo <x ± y>, LLR <value> (<bounds>), time losses <n>
- Disposition: accepted / rejected / neutral / abandoned
- **Calibration (after the result):** <which part of the prediction was
  wrong: sign, size, mechanism, interaction, confidence, or the instrument>
- Artifacts: D:/chess/whitespine-lab/results/WS-NNN/
```

### SPSA

```markdown
### WS-NNN — SPSA: <parameter group>

- Date registered:
- PLAN step:
- Base binary: <commit> / tune build SHA-256 / bench <nodes> of the normal build
- Parameters (name: start, min, max, step):
  - <name>: <start>, <min>, <max>, <step>
- Pilot: <WS-ID of the pilot, or "none" and why>
- Settings: iterations <n>, games per iteration <n>, TC <3+0.03>, book <file>,
  spsa.json a=<>, c=<>, A=<>, alpha=<>, gamma=<>
- **Prediction (frozen):** <which parameters should move and which way; expected
  Elo of the follow-up SPRT>
- Result: final values <...>; parameters that did not move <...>; games played
- Follow-up SPRT: WS-<ID>
- **Calibration:** <...>
- Artifacts: weather-factory folder copy, graph, final config
```

### Texel fit

```markdown
### WS-NNN — Texel fit: <terms>

- Date registered:
- PLAN step:
- Engine commit (trace and parameter layout): <commit>
- Corpus: <name> / manifest SHA-256 / positions train, validation, test
- Free parameters: <groups and count>; fixed: <groups and why>
- Start vector: <current head / neutral>
- Settings: K <value, how fitted>, optimiser <Adam>, learning rate <>, epochs
  <max>, batch <size>, threads <n>
- **Prediction (frozen):** <expected validation-loss change; expected SPRT
  result>
- Result: train loss <start → end>, validation loss <start → best (epoch)>,
  test loss <once, at the end>, largest parameter moves <...>
- Vector: source file SHA-256 <...>; bench of the baked build <nodes>
- Follow-up SPRT: WS-<ID>
- **Calibration:** <...>
```

### Observation

```markdown
### WS-NNN — Observation: <what>

- Date:
- Binary: <commit> / SHA-256 / build profile and features
- Method: <command, repetitions, host state>
- Result: <numbers>
- Meaning and limits: <what this does and does not show>
```

## 3. Worked examples (not real results)

These show how filled entries read. Every number is a placeholder chosen to
look plausible; none was measured.

### WS-EXAMPLE-A — Rewrite gate: 2.0.0 against 1.4.1

- Date registered: `<yyyy-mm-dd>`
- PLAN step: 2.7
- Baseline: 1.4.1 / tag `v1.4.1` / `<sha256>` / no bench command
- Candidate: 2.0.0-rc1 / `<commit>` / `<sha256>` / bench `<nodes>`
- Change and hypothesis: own board, correct quiescence and mate scores, no
  per-node game replay (W-01, W-06, W-12, W-13). All 0.6.1 regression tests
  pass on the candidate. The speed difference alone should be worth a large
  gain.
- Bounds: [0, 10] normalized, alpha 0.05, beta 0.05
- Conditions: TC 8+0.08, Threads 1, Hash not set (1.4.1 has none), book
  UHO_Lichess_4852_v1.epd, concurrency 14, -use-affinity, adjudication off,
  max rounds 100000
- Command: `results/WS-EXAMPLE-A/command.ps1`
- **Prediction (frozen before the first game):**
  - Expected result: accept, probability 0.95
  - Expected size: +150 to +400 Elo
  - Most likely way to be wrong: a time-management bug making 2.0.0 lose on
    time, which would show as time losses rather than as weak play
- Result: games `<n>`, W-D-L `<w-d-l>`, Elo `<x ± y>`, LLR `<value>`, time
  losses `<n>`
- Disposition: `<accepted>`
- **Calibration:** `<for example: sign right, size underestimated because the
  depth gain late in games was larger than the start-position NPS suggested>`
- Artifacts: `D:/chess/whitespine-lab/results/WS-EXAMPLE-A/`

### WS-EXAMPLE-B — Texel fit: the complete seeded evaluation

- PLAN step: 5.4
- Corpus: `ws-data-v1` / manifest `<sha256>` / `<n>` / `<n>` / `<n>` positions
- Free parameters: every linear parameter of the Phase 4 evaluation
  (`<n>`), king-danger weights through the chain rule (`<n>`); fixed and
  excluded: as listed in the fitting manifest `<sha256>` (5.3)
- Start vector: the seeds of Phase 4 (4.15 head)
- Settings: K `<value>` by golden-section search with parameters fixed; Adam,
  learning rate `<value>`, 300 epochs, full batch, 14 threads
- **Prediction (frozen):** validation loss falls by at least `<value>`; the
  follow-up SPRT accepts `[0, 10]`
- Result: validation loss `<start>` → `<best>` at epoch `<n>`; test loss
  `<value>`; mobility tables and king-danger weights moved furthest
- Follow-up SPRT: `WS-<ID>`

### WS-EXAMPLE-C — Observation: perft speed baseline

- Binary: `<commit>` / `<sha256>` / release profile, no features
- Method: `go perft 6` from the start position and `go perft 5` from Kiwipete,
  three runs each, host idle (no match, build or profile running)
- Result: `<n>` million nodes per second with bulk counting, `<n>` without
- Meaning and limits: a reference point for Phase 8; perft speed is not search
  speed and says nothing about strength

## 4. Ledger

Add entries under the phase that owns them, newest last.

### Phase 0 — Foundations and baseline

### Phase 1 — Board

### Phase 2 — Engine rewrite

### Phase 3 — Search fundamentals

### Phase 4 — Classical evaluation

### Phase 5 — Evaluation tuning

### Phase 6 — Advanced search

### Phase 7 — Clock, threads, robustness

### Phase 8 — Performance and classical checkpoint

### Phase 9 — NNUE

## 5. Calibration notes

At each checkpoint (3.13, 4.15, 5.10, 6.10, 7.5, 8.6), read the predictions of the
phase against their results and write down only patterns that repeat: sizes
consistently too optimistic, a kind of change that never works, an instrument
that misled. Do not rewrite the frozen predictions.
