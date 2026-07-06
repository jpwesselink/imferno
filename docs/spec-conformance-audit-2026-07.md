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
| AUDIT-1 | IAB WAVE-PCM false positive | guard `IABEssenceDescriptor` in audio_mca | fixed — PR #69 |
| AUDIT-2 | ISXD edition label | rename :2022 → :2023 (codes + spec_id + docs) | fixed — audit P2 batch (breaking: code prefixes renamed) |
| AUDIT-3 | vendored XSD provenance | swap to canonical + CHECKSUMS | fixed — audit P2 batch |
| AUDIT-4 | ST 2067-204 | scope catalogue (after -203) | backlog |
| AUDIT-5 | ST 377-4 edition | verify §6.3.2 delta in :2021 | Phase 3 |

## Phase 3 — rule-by-rule prose verification (2026-07-02, firsthand-pdf)

Three parallel verification passes against the staged publication PDFs.
All quotes verified against pdftotext extractions; full reports in session
transcripts. Findings numbered on from AUDIT-6.

### ST 2067-2 §5.3/§5.4 + ST 377-4 (audio_mca.rs, timed_text.rs)

- **AUDIT-7 `bug` P1** — `TimedTextMappingKindNot0x13` checks the wrong UL
  byte (`bytes[14]`, should be `bytes[13]`): ST 429-5's canonical container
  UL has 0x13 at byte 14 (index 13) and 0x01 at index 14 → **false Error on
  every conformant IMF timed-text file** (`timed_text.rs:44-63`).
- **AUDIT-8 `bug` P1** — the Mode A carve-out (PR #65) has **no ST 2067-2
  basis**: §5.3.6.2/.3 are unconditional in 2016 AND 2020, and
  ChannelAssignment is mandatory on every audio file (§5.3.4.2 "shall be
  present"), so the gate silently disables §5.3.6 + ST 377-4 checks on any
  file that merely omitted its MCA labels. The Fraunhofer files it was built
  for are ST 2067-204 ADM tracks — -204 §5.1 gates on the ST 2131
  AudioLabelingFrameworkADMContent label and §5.4.1 explicitly bans plain
  ACLSDs. Correct gate: ADM markers (ST 2131 ChannelAssignment value /
  ADMSoundfieldGroupLabelSubDescriptor), mirroring the MGA/IAB gates.
- **AUDIT-9 `bug`** — `MCAChannelIDMissing` false-positives on channel 1:
  Table 7 "may be omitted" when channel ID = 1; § should cite 5.3.6.5.
- **AUDIT-10 `bug`** — `ChannelAssignmentNotMCA` accepts any
  `…04020210.*` UL (Table 5 requires byte 13 = 04h exactly); message
  mis-cites "SMPTE 428-12". Companion `gap`: ChannelAssignment *presence*
  (§5.3.4.2 SHALL) is never checked.
- **AUDIT-11 `bug` (severity)** — `SoundfieldGroupMissingMCA{Title,
  TitleVersion,AudioContentKind,AudioElementKind}`: Table 7 says SHALL for
  the SFG column; emitted as Warning with "recommends" wording.
- **AUDIT-12 `smell`** — `SoundfieldGroupLinkIDMismatch` cites §6.3.2;
  rule lives in ST 377-4 §6.4.1. `MCALinkIDMissing` requirement is Table 1
  "Req"/§5.4. Companion `gap`: per-ACLSD SoundfieldGroupLinkID presence
  (Table 7 SHALL) unchecked.
- **AUDIT-13 edition analysis** — §5.3 numbering/substance identical
  2016↔2020 (pinning :2016 is safe). §5.4 is NOT: 2020 moves to IMSC 1.1,
  adds font/otf (current whitelist → false Error on valid 2020 files),
  adds §5.4.1 DataEssenceCoding prohibition + §5.4.7 root-container rule.
  `TimedTextNamespaceNotIMSC` mixes editions; `TimedTextUCSEncodingNotUTF8`
  mis-attributes to §5.4 (source is ST 429-5).
- **AUDIT-5 RESOLVED** — ST 377-4 §6.3.2/§6.4.1 unchanged 2012→2021
  (clarifications only); :2012 citations remain valid.
- Verified `OK`: SoundDescriptorNotWAVEPCM §5.3.4.1, AudioSampleRate
  §5.3.2.2 (48k/96k IS in 2067-2 prose), QuantizationBits §5.3.2.3,
  clip-wrap §5.3.3, ChannelLabelCount §5.3.6.2, SoundfieldGroupLabelCount
  §5.3.6.3.

### ST 2067-201 IAB (iab.rs, iab_codes.rs)

- **AUDIT-14 `bug` P1** — `MainAudioMissing` (§6.2) is an **invented
  rule**: no edition of -201 (nor ST 2067-2:2020 §6.3.2, "zero or more
  Audio Virtual Tracks") requires a MainAudioSequence alongside an
  IABSequence. Actively firing on user content since the dispatch fix
  (#66); the CC-Namespaces corpus pin added in #66 pinned a false
  positive and must be reverted with the rule.
- **AUDIT-15 `bug`** — `ElectrospatialFormulationForbidden` inverts the
  spec: §5.9 (both editions) "If present … shall be set to a value of 15".
  Presence is legal; check value==15.
- **AUDIT-16 `bug`** — `AudioSamplingRateInvalid` hardcodes 48000/1;
  prose ties the value to the bitstream SampleRate ("48" has zero hits in
  either edition; 96k legal per ST 2098-2). `AudioSamplingRateMissing` is
  a SHALL emitted as Warning.
- **AUDIT-17 `bug` (citations)** — container-label rules cite §5.3
  (→§5.9 Table 4.5); subdescriptor/MCA rules cite §5.9 (→§5.10.2/.3/.4);
  2026 recommendation cites "Annex E §E.2" (→§5.10.2). All UL values
  verified correct.
- **AUDIT-18 `gap`s** — §5.10.2 prohibition of plain
  AudioChannelLabel/SoundfieldGroup/GroupOfSoundfieldGroups subdescriptors;
  "exactly one" IAB SFL upper bound; Annex C.2 MCAChannelID prohibition;
  §6.2 edit-rate integer-multiple-of-Main-Image; Electro-Spatial ==15
  check (replacement for AUDIT-15).
- Verified `OK`: 2019↔2021 edition model (single delta: ChannelCount),
  CodecForbidden, SoundCompression UL, ChannelCountNotZero (2019),
  IABSequenceNoResources. 2026: namespace reuse confirmed; doc comment
  "no normative changes 2021→2026" overclaims (bitstream-level SHALLs
  added) but implemented rule set unaffected.

### ST 2067-202 ISXD + ST 2067-203 S-ADM (isxd.rs, isxd_codes.rs)

- **AUDIT-2 (expanded)** — :2022→:2023 mislabel at ~15 sites incl.
  spec_id, all code prefixes, napi/TS codegen, snapshot tests. Namespace
  `/2022/` confirmed correct per §6 Table 1.
- **AUDIT-19 `bug`** — `SubDescriptorMissing` (Error): "ContainerConstraints"
  appears nowhere in -202 prose; §9.2 says descriptors "may extend" and
  readers "shall ignore unrecognized SubDescriptors". Requirement belongs
  to ST 2127 lineage, not -202 §5.
- **AUDIT-20 `bug`s** — `NamespaceUriMissing`: §5.3/§9.2 SHALL emitted as
  Warning. All 5 emission sites hardcode `Category::Audio`, contradicting
  the catalogue's deliberate `Category::Data`. `NamespaceUriMismatch`
  scoped per-sequence; §6 scopes per-Virtual-Track.
- **AUDIT-21 `gap`s** — §6 ISXD edit rate SHALL equal Main Image edit
  rate (isxd_sequences also missing from the generic per-track edit-rate
  loop at validation/mod.rs:1083-1111); composition-references-ISXD ⇒
  ISXD Virtual Track required; §9.3 DataEssenceCoding UL; §9.1/9.2
  container/descriptor ULs.
- **AUDIT-22 (FIX-16b scoped)** — full ST 2067-203 catalogue extracted:
  15 CPL-level + ~28 MXF-level SHALL/SHOULDs with § cites, mapped 1:1 to
  the 22 test-data/SADM fixtures (map in session transcript / Phase 3
  agent report). `ContainerConstraintsSubDescriptor` and
  `MGAAudioMetadataSubDescriptor` come from ST 2127-1/-10 (not staged) —
  fetch before implementing those two.

### Fix queue additions (priority order)

| # | What | Class |
|---|---|---|
| AUDIT-14 | remove invented MainAudioMissing + revert corpus pin | P1 false positive — **fixed, PR #71** |
| AUDIT-7 | timed-text UL byte index | P1 false positive — **fixed, PR #71** |
| AUDIT-8 | replace Mode A gate with ADM/ST 2131 markers | P1 under-enforcement — **fixed, PR #71** |
| AUDIT-9 | MCAChannelID channel-1 exemption | false positive — **fixed, audit P2 batch** |
| AUDIT-15/16 | IAB electro-spatial + sampling-rate conditions | false positive / over-constraint — **fixed, audit P2 batch** |
| AUDIT-11/20 | SHALL-as-Warning severity corrections | severity — **fixed, audit P2 batch** |
| AUDIT-10/12/17 | § citation corrections + Table 5 UL exactness | citations — **fixed, audit P2 batch** (incl. ChannelAssignment presence + per-ACLSD SoundfieldGroupLinkID presence companion gaps) |
| AUDIT-19 | re-home or delete ISXD SubDescriptorMissing | unsupported rule — **deleted, audit P2 batch** (re-home lands with the ST 2127/-203 catalogue, AUDIT-22) |

Audit P2 batch also closed two AUDIT-21 items: ISXD sequences joined the
generic per-track edit-rate loop, and the §6 ISXD-vs-Main-Image edit-rate
SHALL is now `ST2067-202:2023:6/EditRateMismatch`.

Audit P3 batch (gap rules, all firsthand-pdf):

- **AUDIT-18 closed** — `mxf/iab_labeling.rs`: §5.10.2 prohibition of plain
  ACLSD/SFGLSD/GoSFGLSD (`:5.10.2/ForbiddenMCASubDescriptor`), §5.10.2
  exactly-one IAB SFL (missing + `:5.10.2/SubDescriptorDuplicate`), Annex
  C.2 `:C.2/MCAChannelIDForbidden`; CPL-level §6.2
  `:6.2/EditRateNotIntegerMultiple` (integer multiple of Main Image VT).
- **AUDIT-21 closed (CPL-visible items)** — §6
  `ST2067-202:2023:6/ISXDVirtualTrackMissing` (composition references an
  ISXD Track File without an ISXD VT) and §9.3
  `:9.3/DataEssenceCodingMissing`/`Invalid` (UTF-8 Text Data Essence Coding
  UL, Table 6). §9.1/§9.2 container/descriptor-key ULs remain open: they
  are MXF-set-key level and not visible in the CPL descriptor model.
- **AUDIT-13 (partial)** — `font/otf` accepted for timed-text font
  resources per ST 2067-2:2020 §5.4.6 (whitelist is now the
  union-of-editions; rejecting it false-Errored on valid 2020 files). The
  full §5.4:2020 edition model (IMSC 1.1 namespaces, §5.4.1, §5.4.7)
  remains open.

Still open: AUDIT-4 (-204 plug-in — needs ADMAudioSequence +
ADMAudioVirtualTrackParameterSet CPL parser support; sequenced after -203),
AUDIT-13 remainder (§5.4:2020 edition model), AUDIT-21 remainder (§9.1/§9.2
set-key ULs, MXF level), AUDIT-22 (-203 catalogue; blocked on ST 2127-1/-10
staging).
| AUDIT-2 | :2022→:2023 rename | breaking, own release |
| AUDIT-18/21 | new-rule gaps (IAB + ISXD) | gaps |
| AUDIT-22 | ST 2067-203 catalogue | feature |
