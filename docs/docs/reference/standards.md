---
title: Standards Coverage
description: Which SMPTE ST-2067 standards imferno implements and to what degree.
---

| Standard | Title | Editions | Coverage |
|---|---|---|---|
| ST 429-9 | Volume Index / Asset Map | 2014 | Complete |
| ST 2067-2 | Core Constraints and Packing List | 2013 / 2016 / 2020 | Complete |
| ST 2067-3 | Composition Playlist | 2013 / 2016 / 2020 | Complete |
| ST 2067-21 | Application #2E (UHD/HDR) | 2020 / 2023 / 2025 | Complete |
| ST 2067-201 | IAB Level 0 Plug-in | 2019 / 2021 / 2026 | Complete (2026 adds Annex E `IABChannelSubDescriptor` recommendation) |
| ST 377-1 | MXF File Format | 2011 | Partial — full header metadata via RegXML (Preface, MaterialPackage, EssenceDescriptors, sub-descriptors) read from footer with header fallback; KLV is traversed to locate metadata sets but essence samples are not decoded |
| ST 2067-9 | Sidecar Composition Map | 2018 | Complete |
| ST 429-8 | D-Cinema Packing List | 2007 | Not implemented |
| ST 2067-100 | Output Profile List | 2014 | Not implemented |
| ST 2067-202 | ISXD Plug-in | 2022 | Complete |
| ST 2067-203 | S-ADM Audio Plug-in | 2023 | Partial — MGA/S-ADM audio descriptors detected via `MGASoundEssenceDescriptor`; ST 2067-2 §5.3 WAVE-PCM-only rules correctly stand down. Verified against the Fraunhofer SMPTE working-group corpus (2 S-ADM audio tracks parse clean). CPL-level plug-in semantics (SADM VirtualTrack, MGASoundfield labeling, ADM metadata structure) not yet enforced. |
| ST 2067-204 | ADM Audio Plug-in | 2023 | Partial — ADM audio detected via ST 2131 `ChannelAssignment` label (bytes 9–13 = `04.02.02.10.05`) or `ADM*SubDescriptor` elements; §5.4.1 prohibition on plain MCA sub-descriptors respected. Verified against the Fraunhofer SMPTE working-group corpus (2 ADM audio tracks parse clean bar an RFC 5646 language-tag recommendation). CPL-level plug-in semantics not yet enforced. |
| ST 377-41 | MXF MGA / S-ADM Virtual Tracks | — | Not implemented |
| ST 379-2 | MXF Generic Container | 2010 | Not implemented |
| ST 422 | JPEG 2000 in MXF | 2014 | Not implemented |

Standards not listed are not implemented and not on the roadmap. The "Not implemented" entries above are the recognised gaps relative to the broader IMF ecosystem.

## Known gaps

- MXF header metadata (Preface → MaterialPackage → EssenceDescriptors → sub-descriptors) is fully parsed via RegXML; the body partition is not byte-decoded — no frame-level JPEG 2000 / WAVE PCM / IAB sample inspection
- Multi-volume packages (volumeIndex > 1) parse but are not fully validated
- Output Profile List (OPL) transformation is not implemented

## Test corpus

Tests run against a vendored IMF corpus covering: App2, App2E 2020/2021, App5 (IMAX), IAB, ISXD, HT (JPEG 2000 High Throughput), MERIDIAN.

The essence-layer ST 2067-203 (S-ADM) and ST 2067-204 (ADM) rules are
additionally validated against the Fraunhofer SMPTE working-group test
corpus (10 MXFs, CC-BY-NC-ND 4.0, not vendored). Fetch it with
`scripts/fetch-fraunhofer-corpus.sh` and drive the essence checks with
`cargo run --example fraunhofer_corpus_check -- test-data/Fraunhofer-SMPTE-ST2067-203-204`.
