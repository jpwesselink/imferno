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
| ST 377-1 | MXF File Format | 2011 | Complete — KLV traversal, partition packs (header + footer with fallback), full header metadata via RegXML (Preface → MaterialPackage → EssenceDescriptors → sub-descriptors) using the SMPTE Elements/Groups/Types dictionaries, essence container UL detection. (Decoding essence samples into codec output is out of scope for ST 377-1 itself — see the codec rows below.) |
| ST 377-4 | MXF MCA (Multi-Channel Audio) Labeling | 2012 | Complete — MCALinkID, SoundfieldGroupLinkID, MCAChannelID coverage per §6.3–§6.4; ADM/S-ADM audio correctly bypasses the plain-MCA rules per ST 2067-204 §5.4.1. |
| ST 2067-9 | Sidecar Composition Map | 2018 | Complete |
| ST 429-8 | D-Cinema Packing List | 2007 | Not implemented |
| ST 2067-100 | Output Profile List | 2014 | Not implemented |
| ST 2067-202 | ISXD Plug-in | 2022 | Complete |
| ST 2067-203 | S-ADM Audio Plug-in | 2023 | Complete — essence-layer MGA/S-ADM detection (WAVE-PCM rules correctly stand down) plus CPL-level §5.3 / §5.4 plug-in semantics: `MGASADMSignalSequence` parsed as first-class sequence, `MGASADMVirtualTrackParameterSet` cross-referenced via `TrackId`, orphaned-parameter-set + missing-parameter-set + empty-operational-mode + orphaned-SoundfieldGroupSelector.ResourceId all enforced. Verified against the Fraunhofer SMPTE working-group corpus. |
| ST 2067-204 | ADM Audio Plug-in | 2023 | Complete — mirror of the ST 2067-203 implementation: essence-layer ADM detection via ST 2131 `ChannelAssignment` label (bytes 9–13 = `04.02.02.10.05`) or `ADM*SubDescriptor` elements, plus CPL-level `ADMAudioSequence` + `ADMAudioVirtualTrackParameterSet` §5.4 cross-reference enforcement. Verified against the Fraunhofer SMPTE working-group corpus. |
| ST 377-41 | MXF MGA / S-ADM Virtual Tracks | — | Not implemented |
| ST 379-2 | MXF Generic Container | 2010 | Not implemented |
| ST 422 | JPEG 2000 in MXF | 2014 | Not implemented |

Standards not listed are not implemented and not on the roadmap. The "Not implemented" entries above are the recognised gaps relative to the broader IMF ecosystem.

## Known gaps

- Essence samples inside MXF body partitions are not decoded — that's the domain of the codec-specific specs (**ST 422** JPEG 2000 in MXF, **ST 382** AES-3 audio in MXF, **ST 379-2** Generic Container), all listed as "Not implemented" above. ST 377-1 structural validation (KLV, partitions, metadata) is complete; a JPEG 2000 or PCM byte-level decoder is a separate concern
- Multi-volume packages (volumeIndex > 1) parse but are not fully validated
- Output Profile List (OPL) transformation is not implemented
- Deep MXF ↔ CPL cross-references for ST 2067-203/-204 (matching a `MGASADMSoundfieldGroupSelector.MGASoundfieldGroupLinkID` to the target MXF's `MGASoundfieldGroupLabelSubDescriptor.MCALinkID`) are not yet enforced — the CPL-side selector→resource cross-check is complete, but the descriptor-side link isn't verified against the MXF header metadata

## Test corpus

Tests run against a vendored IMF corpus covering: App2, App2E 2020/2021, App5 (IMAX), IAB, ISXD, HT (JPEG 2000 High Throughput), MERIDIAN.

The essence-layer ST 2067-203 (S-ADM) and ST 2067-204 (ADM) rules are
additionally validated against the Fraunhofer SMPTE working-group test
corpus (10 MXFs, CC-BY-NC-ND 4.0, not vendored). Fetch it with
`scripts/fetch-fraunhofer-corpus.sh` and drive the essence checks with
`cargo run --example fraunhofer_corpus_check -- test-data/Fraunhofer-SMPTE-ST2067-203-204`.
