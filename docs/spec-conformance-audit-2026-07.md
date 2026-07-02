# Spec-conformance audit — 2026-07 (audio stack)

Living document. Cross-checks imferno's validation rules against SMPTE spec
text in both directions: **incorrect** (rule condition / severity / section /
edition wrong), **missing** (SHALL statement with no rule), **over-applied**
(rule fires outside its spec's scope). Companion to the parser audit
(`parser-audit-2026-06.md`), which covered parsers rather than rules.

Scope of this pass (user decision): the audio stack — ST 2067-2 §5.3/§5.4,
ST 377-4, ST 2067-201 (IAB), ST 2067-202 (ISXD), ST 2067-203 (S-ADM),
ST 2067-204 (ADM). Core constraints / App2E image rules are a later pass.

## Findings legend

Same as the parser audit: `bug` (wrong behavior, ship a fix + regression
test), `smell` (works but fragile/misleading), `gap` (spec requirement with
no rule), `OK` (verified correct). Confidence tags: `firsthand-pdf`,
`firsthand-xsd`, `secondhand-comparison-doc`, `cross-reference-tool`,
`corpus-behavior`.

## Source material acquired (2026-07-02)

All fetched from pub.smpte.org (free publication copies), staged in
`/tmp/spec-staging/`. Re-fetch: `curl -sLO https://pub.smpte.org/doc/<doc>/<date>-pub/<file>`.

| Spec | Edition | File | sha256 (16) |
|---|---|---|---|
| ST 2067-2 | 2016 | st2067-2-2016.pdf | `5f9be51f3e4fa249…` |
| ST 2067-2 | 2020 | st2067-2-2020.pdf | `65d291ca0e9d5a26…` |
| ST 2067-201 | 2019 | st2067-201-2019.pdf | `a00f85716a6e4ad3…` |
| ST 2067-201 | 2021 | st2067-201-2021.pdf | `8cdc81c2056848b4…` |
| ST 2067-201 | 2026 | SMPTE-ST-2067-201-2026-03-25-…pdf (+ HTML) | — |
| ST 2067-202 | **2023** | st2067-202-2023.pdf + st2067-202a-2023.xsd | `3b03ae6ccc24e43b…` |
| ST 2067-203 | 2023 | st2067-203-2023.pdf + st2067-203a-2023.xsd | `89fd632445ecf048…` |
| ST 2067-204 | 2026-05 | st2067-204-2026-05.pdf + **canonical XSD** + **official sample MXF** | `d0625f9a10e149e5…` |
| ST 377-4 | 2012 | st0377-4-2012.pdf | `523bc3bd99399d53…` |
| ST 377-4 | **2021** | st377-4-2021.pdf | `ecfbfba305646f35…` |

Also available: the Fraunhofer SMPTE WG ST 2067-203/-204 test corpus
(IMFTool repo, CC-BY-NC-ND, 10 MXFs — do not vendor).

## Findings

### AUDIT-0 — `bug` (FIXED, PR #66) — IAB/ISXD plugins unreachable in production

`AppIabPlugin2019/2021/2026` and `AppIsxdPlugin2022` were implemented and
tested but registered in no validator registry; nothing dispatched on
sequence presence either. `imferno validate` on IAB/ISXD packages ran zero
plug-in rules. Fixed by URI registration + sequence-presence dispatch
(`push_sequence_presence_plugins`), 7 regression tests in
`tests/plugin_dispatch.rs`. Confidence: `corpus-behavior` + source
inspection. Corpus effect: CC-Namespaces fixture now correctly fires
ST 2067-201 §6.2 `MainAudioMissing` (true positive; pinned in corpus test).

### AUDIT-1 — `bug` (OPEN) — `SoundDescriptorNotWAVEPCM` false positive on IAB essence

`mxf/audio_mca.rs` skip-guards `MGASoundEssenceDescriptor` (SADM, fixed in
v3.0.1) but not `IABEssenceDescriptor`. ST 2067-201 opens the IAB essence
path exactly as -203 opened MGA; an IAB Atmos MXF
(`test-data/IAB/CompleteIMP/meridian_2398_Atmos_…mxf`) fires
`ST2067-2:2016:5.3.4.1/SoundDescriptorNotWAVEPCM` — a false positive on
every IAB package. Fix: extend the essence-type guard to
`IABEssenceDescriptor` (verify descriptor name against ST 2067-201 §5.9 +
RegXML emission). Confidence: `corpus-behavior`; descriptor naming to be
pinned `firsthand-pdf` during Phase 3. **Land after PR #65** (audio_mca.rs
conflict).

### AUDIT-2 — `bug` (OPEN) — ISXD edition mislabeled :2022 (spec is :2023)

Title page of the publication PDF reads **"SMPTE ST 2067-202:2023"**
(`firsthand-pdf`, sha `3b03ae6c…`). imferno labels everything :2022:
`AppIsxdPlugin2022` spec_id "ST 2067-202:2022 (ISXD Plug-in)",
`St2067_202_2022` enum, `ST2067-202:2022:*` code prefix, standards tables
in README/docs. The *namespace URI* `…/ns/2067-202/2022` is correct — SMPTE
froze the namespace year at 2022 while the edition year is 2023 (the -203
and -204 namespaces are also `/2022/`). Fix: rename edition labels to
:2023, keep URI_2022; codes are operator-visible strings so this is a
breaking change for rule configs — coordinate with a minor release.

### AUDIT-3 — `smell` — vendored -202/-203 XSDs differ from canonical copies

Vendored copies (Photon-sourced) add `schemaLocation` hints to
`<xs:import>` directives pointing at Photon-bundle-relative paths that
don't exist in our tree; canonical publication XSDs have namespace-only
imports. No semantic constraint difference (verified by diff). Action:
replace vendored copies with the canonical publication XSDs + update
`specs/CHECKSUMS` so the drift checker tracks the true upstream. Also
closes parser-audit J2/J3 (the 3 plugin XSDs previously had no known
canonical URL — they do now: the pub.smpte.org zips).

### AUDIT-4 — `gap` — ST 2067-204 exists (2026-05) with zero imferno support

Published 2026-05-27: "IMF — Audio with ADM Metadata Plug-in", with a
canonical XSD (`st2067-204a-2026-05.xsd`, targetNamespace
`…/ns/2067-204/2022`) and an official sample MXF in the publication zip.
Fraunhofer test vectors exist. imferno: no parser support for
ADMAudioSequence(?), no rules, standards tables say "Not implemented"
(correct today). Action: scope a -204 catalogue after the -203 one.

### AUDIT-5 — `question` — ST 377-4 citation currency (2012 vs 2021)

A 2021 edition of ST 377-4 (MCA) exists; imferno's codes cite
`ST377-4:2012:6.3.2/*`. Phase 3 must check whether §6.3.2's MCALinkID
rules changed in 2021 and whether codes should cite the newer edition.

### AUDIT-6 — `note` — ST 2067-203/-204 namespace year is /2022/

All three plug-in namespaces (`2067-202`, `2067-203`, `2067-204`) use
`…/ns/<part>/2022` regardless of edition year. Relevant for future
dispatch arms (the -203 sequence namespace to match is
`http://www.smpte-ra.org/ns/2067-203/2022`).

## Phase status

- [x] Phase 0.1 — Fraunhofer branch landed (PR #65: Mode A labeling,
      partition fallback)
- [x] Phase 0.2 — P0 dispatch fix landed (PR #66)
- [x] Phase 1 — prose acquired for all audio-stack specs (table above);
      manifest extension pending
- [ ] Phase 2 — comparison docs (st2067-201/202/203/204, st377-4)
- [ ] Phase 3 — rule-by-rule forward pass (audio_mca, timed_text, iab,
      isxd rules vs prose)
- [ ] Phase 4 — SHALL-statement reverse pass
- [ ] Phase 5 — consolidate + fix tickets

## Fix queue

| # | Finding | Action | Status |
|---|---|---|---|
| AUDIT-0 | dispatch gap | PR #66 | merged |
| AUDIT-1 | IAB WAVE-PCM false positive | guard `IABEssenceDescriptor` in audio_mca | open — after #65 |
| AUDIT-2 | ISXD edition label | rename :2022 → :2023 (codes + spec_id + docs) | open — breaking, coordinate release |
| AUDIT-3 | vendored XSD provenance | swap to canonical + CHECKSUMS | open |
| AUDIT-4 | ST 2067-204 | scope catalogue (after -203) | backlog |
| AUDIT-5 | ST 377-4 edition | verify §6.3.2 delta in :2021 | Phase 3 |
