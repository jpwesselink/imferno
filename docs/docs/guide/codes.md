---
title: Validation Codes
description: Every validation rule imferno checks, per spec and per edition.
---

Every finding names the rule it reports and the edition of the spec that was
applied, as a code like `ST2067-3:2016:7.2/SegmentDuration`. Use the rule
ids and codes to [configure rule severity](/guide/config/).

## Code format

```
<SPEC>:<EDITION>:<CLAUSE>/<Name>
```

| Part | Meaning |
|------|---------|
| `SPEC` | SMPTE document, e.g. `ST2067-2`, `ST429-9`. `IMFERNO` for engine checks with no SMPTE source; `XSD` for schema validation. |
| `EDITION` | The edition of the spec that was **applied** (`2013`, `2016`, `2020`, …). For `XSD` the schema used (`CPL-2016`); omitted for `IMFERNO`. |
| `CLAUSE` | Clause of that edition the rule comes from (`7.2.2`, `A.2`). For `IMFERNO` a group (`Package`, `Mxf`). |
| `Name` | The rule's name, unique within the spec. |

Each rule has a stable **rule id** `SPEC/Name` (e.g. `ST2067-3/SegmentDuration`)
that does not change when a clause is corrected; the clause and severity of
each edition live in the rule's edition table below.

In the tables, each edition column shows the clause and default severity.
`—` means the edition does not contain the rule. `✓` marks a row checked
against that edition's text; `†` a clause known or suspected to be wrong.
Unmarked rows are carried over from earlier catalogues and not yet checked.

## SMPTE ST 377-1 — MXF File Format

| Rule | Description | Category | 2011 | 2019 |
|------|-------------|----------|------|------|
| `ST377-1/NotMxf` | File is not a valid MXF container. | Asset | 6.1.1✓ error | 6.1.1✓ error |
| `ST377-1/NoEssenceContainers` | MXF file contains no essence containers. | Encoding | 7.1✓ warning | 7.1✓ warning |
| `ST377-1/NonHeaderFirstPartition` | The first partition in an MXF file must be the Header partition. | Container | 6.2.5✓ error | 6.2.5✓ error |
| `ST377-1/HeaderPartitionOpen` | The header partition is Open and no Closed footer partition carries header metadata, so the MXF file is Open. | Container | 6.2.4✓ warning | 6.2.4✓ warning |
| `ST377-1/MissingHeaderMetadata` | The header partition must carry header metadata (HeaderByteCount > 0). | Container | 6.1.1✓ error | 6.1.1✓ error |
| `ST377-1/StructuralComponentDurationMissing` | A Structural Component on a Timeline Track has no Duration; it is a Best Effort property there and shall be encoded (-1 when unknown). | Container | B.8✓ error | B.8✓ error |

## SMPTE ST 377-4 — MXF Multichannel Audio Labeling

| Rule | Description | Category | 2012 | 2021 |
|------|-------------|----------|------|------|
| `ST377-4/MCALinkIDMissing` | Every MCA sub-descriptor shall carry an MCALinkID (§6.3 Table 3, Req). | Audio | 6.3✓ error | 6.3✓ error |
| `ST377-4/SoundfieldGroupLinkIDMismatch` | AudioChannelLabelSubDescriptor SoundfieldGroupLinkID shall equal the SoundfieldGroup's MCALinkID (§6.4.1). | Audio | 6.4.1✓ error | 6.4.1✓ error |

## SMPTE ST 429-8 — Packing List

| Rule | Description | Category | 2007 |
|------|-------------|----------|------|
| `ST429-8/SizeMismatch` | The asset's size differs from the Size element the Packing List declares for it. | Asset | 6.4✓ error |
| `ST429-8/ChecksumMismatch` | The asset's SHA-1 message digest differs from the Hash element the Packing List declares for it. | Asset | 6.3✓ error |

## SMPTE ST 429-9 — Asset Mapping and File Segmentation

| Rule | Description | Category | 2014 |
|------|-------------|----------|------|
| `ST429-9/AssetListEmpty` | The Asset Map's AssetList contains no Asset element (the §11 schema requires one or more). | Structure | 11✓ critical |
| `ST429-9/VolindexMissing` | No volume-index document found in the package root. | Structure | 8✓ info |
| `ST429-9/MalformedXml` | The VOLINDEX.xml document is not well-formed XML. | Structure | 8✓ warning |
| `ST429-9/PathNotRelative` | Asset Map Path is not a relative-path reference without query or fragment (Basic Map Profile v2). | Structure | A.2✓ error |
| `ST429-9/PathCharacters` | Asset Map Path segment uses characters other than a-z, A-Z, 0-9, dash, underscore and period (Basic Map Profile v2). | Structure | A.2✓ error |
| `ST429-9/PathTooLong` | Asset Map Path or one of its segments is longer than 100 characters (Basic Map Profile v2). | Structure | A.2✓ error |
| `ST429-9/PathTooManySegments` | Asset Map Path has more than 10 segments (Basic Map Profile v2). | Structure | A.2✓ error |
| `ST429-9/PathDuplicate` | Two Asset Map Paths are identical regardless of case (Basic Map Profile v2). | Structure | A.2✓ error |
| `ST429-9/AssetMapFileName` | The Asset Map file is not named exactly ASSETMAP.xml (Basic Map Profile v2). | Structure | A.4✓ error |
| `ST429-9/AssetMapHasNoPackingList` | AssetMap declares no asset as a PackingList (ST 429-9 §6.3). | Structure | 6.3✓ critical |
| `ST429-9/PklIdNotInAssetMap` | PKL document Id is not declared as a PackingList asset in the AssetMap. | Reference | 6.3✓ error |
| `ST429-9/AssetMapMalformedXml` | The ASSETMAP.xml document is not well-formed XML. | Structure | 11✓ error |

## SMPTE ST 2067-2 — Core Constraints

| Rule | Description | Category | 2013 | 2016 | 2020 |
|------|-------------|----------|------|------|------|
| `ST2067-2/NotImfCompositionPlaylist` | The CPL is not an IMF Composition Playlist: its namespace is not an ST 2067-3 namespace (e.g. a D-Cinema ST 429-7 CPL). No ST 2067-3, Core Constraints or application rules are applied to it. | Structure | 6.1✓ error | 6.1✓ error | 6.1✓ error |
| `ST2067-2/PklMalformedXml` | A Packing List document is not well-formed XML. | Structure | 7.1✓ error | K✓ error | J✓ error |
| `ST2067-2/NoCpls` | No CPL assets found in the AssetMap. | Structure | —✓ | 7.2.1✓ info | 7.2✓ info |
| `ST2067-2/SizeMismatch` | Declared file size does not match the on-disk size. | Asset | —✓ | K✓ error | J✓ error |
| `ST2067-2/FileNotFound` | A referenced asset file is not present at the declared path. | Asset | 7.1✓ error | 7.2.1✓ error | 7.2✓ error |
| `ST2067-2/ChecksumMismatch` | File hash does not match the declared SHA-1/SHA-256 checksum. | Asset | —✓ | K✓ error | J✓ error |
| `ST2067-2/UnresolvedUuid` | An asset listed in the Packing List is not in the AssetMap. | Reference | —✓ | 8.3✓ warning | 8.3✓ warning |
| `ST2067-2/TrackFileNotInPackage` | A CPL track file is not in this package; an IMP need not contain complete compositions (e.g. a supplemental package referring to the original version). | Reference | 7.6✓ info | 7.2.3✓ info | 7.2✓ info |
| `ST2067-2/DuplicateUuid` | Two or more assets within the package share the same UUID. | Reference | 7.2✓ error | 7.3.1✓ error | 7.3.1✓ error |
| `ST2067-2/PklUnknownNamespace` | PKL document namespace is not one of the published SMPTE PKL namespaces. | Structure | 7.1✓ error | K✓ error | J✓ error |
| `ST2067-2/VirtualTrackEditRate` | All resources in a virtual track shall have the same edit rate. | Timing | 6.3✓ error | 6.3✓ error | 6.3✓ error |
| `ST2067-2/TimedTextSampleRate` | DCTimedTextDescriptor SampleRate is missing. | Subtitle | 5.4.2✓ error | 5.4.2✓ error | 5.4.2✓ error |
| `ST2067-2/TimedTextEmptyLanguageTag` | Empty language tag in RFC5646LanguageTagList. | Subtitle | 5.4.4✓ error | 5.4.4✓ error | 5.4.4✓ error |
| `ST2067-2/TimedTextMalformedLanguageTag` | Language tag does not start with an ASCII letter (RFC 5646 primary subtag). | Subtitle | 5.4.4✓ error | 5.4.4✓ error | 5.4.4✓ error |
| `ST2067-2/AudioSampleRate` | WAVEPCMDescriptor has no AudioSampleRate or SampleRate. | Audio | 5.3.4.1✓ error | 5.3.4.1✓ error | 5.3.4.1✓ error |
| `ST2067-2/ChannelCount` | WAVEPCMDescriptor ChannelCount is missing. | Audio | 5.3.4.1✓ error | 5.3.4.1✓ error | 5.3.4.1✓ error |
| `ST2067-2/ChannelCountZero` | WAVEPCMDescriptor ChannelCount is zero. | Audio | 5.3.2.1✓ error | 5.3.2.1✓ error | 5.3.2.1✓ error |
| `ST2067-2/MCASubDescriptors` | WAVEPCMDescriptor has no MCA SubDescriptors. | Audio | 5.3.6.2✓ error | 5.3.6.2✓ error | 5.3.6.2✓ error |
| `ST2067-2/SoundfieldGroup` | WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor. | Audio | 5.3.6.3✓ error | 5.3.6.3✓ error | 5.3.6.3✓ error |
| `ST2067-2/MCATagSymbol` | SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol. | Audio | 5.3.6.1✓ error | 5.3.6.1✓ error | 5.3.6.1✓ error |
| `ST2067-2/SoundfieldChannelCount` | WAVEPCMDescriptor.ChannelCount exceeds the number of channels of its soundfield group (a subset is allowed). | Audio | 5.3.6.6✓ error | 5.3.6.6✓ error | 5.3.6.6✓ error |
| `ST2067-2/DigitalSignature` | The CPL carries a Signature (ST 2067-2 §6.9); signature verification is not performed. | Security | 6.9✓ info | 6.9✓ info | 6.9✓ info |
| `ST2067-2/EssenceDescriptorList` | EssenceDescriptorList is absent: the File Descriptor of every Track File the CPL references shall be mapped to an EssenceDescriptor (§6.8). | Structure | 6.8✓ error | 6.8✓ error | 6.8✓ error |
| `ST2067-2/CplIdAssetMapMismatch` | An AssetMap asset points at a CPL file whose Id differs from the asset Id. | Reference | 7.2✓ error | 7.3.1✓ error | 7.3.1✓ error |
| `ST2067-2/OperationalPatternNotOP1A` | MXF essence Operational Pattern UL is not OP1a (ST 378:2004). | Container | 5.1.1✓ error | 5.1.1✓ error | 5.1.1✓ error |
| `ST2067-2/SoundDescriptorNotWAVEPCM` | Audio essence descriptor must be WAVEPCMDescriptor. | Audio | 5.3.4.1✓ error | 5.3.4.1✓ error | 5.3.4.1✓ error |
| `ST2067-2/AudioSampleRateUnsupported` | AudioSampleRate must be 48000 Hz or 96000 Hz. | Audio | 5.3.2.2✓ error | 5.3.2.2✓ error | 5.3.2.2✓ error |
| `ST2067-2/QuantizationBitsNot24` | QuantizationBits must be 24. | Audio | 5.3.2.3✓ error | 5.3.2.3✓ error | 5.3.2.3✓ error |
| `ST2067-2/ChannelLabelCountMismatch` | Number of AudioChannelLabelSubDescriptors must equal ChannelCount. | Audio | 5.3.6.2✓ error | 5.3.6.2✓ error | 5.3.6.2✓ error |
| `ST2067-2/MCAChannelIDMissing` | MCAChannelID shall be present per channel (channel 1 may omit it) — §5.3.6.5 Table 7. | Audio | —✓ | 5.3.6.5✓ error | 5.3.6.5✓ error |
| `ST2067-2/SoundFieldGroupLabelCount` | Exactly one SoundfieldGroupLabelSubDescriptor is required per WAVEPCMDescriptor. | Audio | 5.3.6.3✓ error | 5.3.6.3✓ error | 5.3.6.3✓ error |
| `ST2067-2/SoundfieldGroupLinkIDMissing` | Every AudioChannelLabelSubDescriptor shall carry a SoundfieldGroupLinkID (§5.3.6.5 Table 7). | Audio | 5.3.6.5✓ error | 5.3.6.5✓ error | 5.3.6.5✓ error |
| `ST2067-2/AudioNotClipWrapped` | Audio essence must be Wave Clip-Wrapped (ST 382 §10). | Container | 5.3.3✓ error | 5.3.3✓ error | 5.3.3✓ error |
| `ST2067-2/RFC5646SpokenLanguageMissing` | Audio descriptor should carry an RFC-5646 spoken language tag. | Audio | 5.3.6.5✓ warning | 5.3.6.5✓ warning | 5.3.6.5✓ warning |
| `ST2067-2/ChannelAssignmentMissing` | ChannelAssignment shall be present on the audio essence descriptor (§5.3.4.2). | Audio | 5.3.4.2✓ error | 5.3.4.2✓ error | 5.3.4.2✓ error |
| `ST2067-2/ChannelAssignmentNotMCA` | ChannelAssignment shall equal the ST 2067-2 Table 5 channel-assignment label UL. | Audio | 5.3.4.2✓ error | 5.3.4.2✓ error | 5.3.4.2✓ error |
| `ST2067-2/SoundfieldGroupMissingMCATitle` | SoundfieldGroupLabelSubDescriptor shall carry MCATitle (§5.3.6.5 Table 7). | Audio | 5.3.6.5✓ error | 5.3.6.5✓ error | 5.3.6.5✓ error |
| `ST2067-2/SoundfieldGroupMissingMCATitleVersion` | SoundfieldGroupLabelSubDescriptor shall carry MCATitleVersion (§5.3.6.5 Table 7). | Audio | 5.3.6.5✓ error | 5.3.6.5✓ error | 5.3.6.5✓ error |
| `ST2067-2/SoundfieldGroupMissingMCAAudioContentKind` | SoundfieldGroupLabelSubDescriptor shall carry MCAAudioContentKind (§5.3.6.5 Table 7). | Audio | 5.3.6.5✓ error | 5.3.6.5✓ error | 5.3.6.5✓ error |
| `ST2067-2/SoundfieldGroupMissingMCAAudioElementKind` | SoundfieldGroupLabelSubDescriptor shall carry MCAAudioElementKind (§5.3.6.5 Table 7). | Audio | 5.3.6.5✓ error | 5.3.6.5✓ error | 5.3.6.5✓ error |
| `ST2067-2/TimedTextUCSEncodingNotUTF8` | TimedTextDescriptor UCSEncoding must be UTF-8. | Subtitle | 5.4.2✓ error | 4.3✓ error | 4.3✓ error |
| `ST2067-2/TimedTextNamespaceNotIMSC` | TimedTextDescriptor NamespaceURI must be one of the IMSC1 profile namespaces. | Subtitle | —✓ | 5.4.2✓ error | 5.4.2✓ error |
| `ST2067-2/TimedTextResourceMIMETypeUnsupported` | TimeTextResourceSubDescriptor MIMEType must be image/png, font/otf (2020) or application/x-font-opentype. | Subtitle | 5.4.5✓ error | 5.4.5✓ error | 5.4.5✓ error |
| `ST2067-2/TimedTextMappingKindNot0x13` | Timed-text Essence Container UL Mapping Kind byte (byte 14) must be 13h, Timed Text (ST 429-5 Annex A Table A.1). | Container | 5.4.2✓ error | 5.4.2✓ error | 5.4.2✓ error |
| `ST2067-2/MainImageVirtualTrackCount` | A Composition shall contain exactly one Main Image Virtual Track. | Structure | 6.3.1✓ error | 6.3.1✓ error | 6.3.1✓ error |
| `ST2067-2/AudioVirtualTrackMissing` | A Composition shall contain one or more Audio Virtual Tracks (ST 2067-2:2013 and 2016; 2020 allows zero). | Structure | 6.3.2✓ error | 6.3.2✓ error | —✓ |

## SMPTE ST 2067-3 — Composition Playlist

| Rule | Description | Category | 2013 | 2016 | 2020 |
|------|-------------|----------|------|------|------|
| `ST2067-3/ContentTitleEmpty` | ContentTitle is empty; it shall contain a human-readable title. | Metadata | 6.1.7✓ error | 6.1.7✓ error | 6.1.7✓ error |
| `ST2067-3/ResourceListEmpty` | A Sequence has an empty ResourceList; the SequenceType schema requires one or more Resource elements. | Structure | 6.10✓ error | 6.10✓ error | 6.10✓ error |
| `ST2067-3/ContentKindUnknown` | ContentKind uses an unrecognized value under the SMPTE scope. | Metadata | 6.1.8✓ error | 6.1.8✓ error | 6.1.8✓ error |
| `ST2067-3/SourceEncodingNoEssenceDescriptorList` | SourceEncoding present but EssenceDescriptorList absent. | Reference | 6.12.1✓ error | 6.12.1✓ error | 6.12.1✓ error |
| `ST2067-3/SourceEncodingUnresolved` | SourceEncoding does not match any EssenceDescriptor Id. | Reference | 6.12.1✓ error | 6.12.1✓ error | 6.12.1✓ error |
| `ST2067-3/EssenceDescriptorListEmpty` | EssenceDescriptorList present but contains no descriptors. | Structure | 6.1✓ error | 6.1✓ error | 6.1✓ error |
| `ST2067-3/ContentVersionListEmpty` | ContentVersionList present but empty. | Structure | 6.1✓ error | 6.1✓ error | 6.1✓ error |
| `ST2067-3/ContentVersionIdInvalid` | ContentVersion/Id is empty (shall be a URI). | Metadata | 6.8.1✓ error | 6.8.1✓ error | 6.8.1✓ error |
| `ST2067-3/ContentVersionLabelTextMissing` | ContentVersion/LabelText is absent. | Metadata | 6.8✓ error | 6.8✓ error | 6.8✓ error |
| `ST2067-3/LocaleLanguageTagInvalid` | Locale language tag does not conform to RFC 5646. | Metadata | 6.5.2✓ error | 6.5.2✓ error | 6.5.2✓ error |
| `ST2067-3/TrackIdNotUnique` | TrackId is not unique within a segment. | Structure | 6.9.3✓ error | 6.9.3✓ error | 6.9.3✓ error |
| `ST2067-3/MarkerOffsetOutOfRange` | Marker offset exceeds the resource IntrinsicDuration. | Timing | 6.13✓ error | 6.13✓ error | 6.13✓ error |
| `ST2067-3/MarkerLabelUnknown` | Marker label is not a recognized SMPTE standard value. | Metadata | 6.14.2✓ error | 6.14.1.2✓ error | 6.14.1.2✓ error |
| `ST2067-3/SegmentDuration` | All virtual tracks in a segment must span the same number of edit units. | Timing | 7.2✓ error | 7.2✓ error | 7.2✓ error |
| `ST2067-3/ContentVersionIdDuplicate` | No two ContentVersion elements shall have identical Id values. | Structure | 6.1.9✓ error | 6.1.9✓ error | 6.1.9✓ error |
| `ST2067-3/SegmentDurationIntegerEditUnits` | Sequence duration shall be an integer number of Composition Edit Units. | Timing | 7.3✓ error | 7.3✓ error | 7.3✓ error |
| `ST2067-3/UniqueSegmentId` | Duplicate Segment Id within the CPL. | Structure | 6.9.1✓ error | 6.9.1✓ error | 6.9.1✓ error |
| `ST2067-3/UniqueEssenceDescriptorId` | Duplicate EssenceDescriptor Id within the CPL. | Structure | 6.7.1✓ error | 6.7.1✓ error | 6.7.1✓ error |
| `ST2067-3/UniqueResourceId` | Duplicate Resource Id within the CPL. | Structure | 6.11.1✓ error | 6.11.1✓ error | 6.11.1✓ error |
| `ST2067-3/EntryPoint` | EntryPoint exceeds IntrinsicDuration, leaving no valid SourceDuration. | Timing | 6.11.6✓ error | 6.11.6✓ error | 6.11.6✓ error |
| `ST2067-3/ResourceDuration` | EntryPoint + SourceDuration exceeds IntrinsicDuration. | Timing | 6.11.6✓ error | 6.11.6✓ error | 6.11.6✓ error |
| `ST2067-3/RepeatCount` | RepeatCount shall be a positive integer. | Timing | 6.11✓ error | 6.11✓ error | 6.11✓ error |
| `ST2067-3/TrackFileId` | A non-marker resource is missing a TrackFileId. | Reference | 6.12✓ error | 6.12✓ error | 6.12✓ error |
| `ST2067-3/VirtualTrackContinuity` | A virtual track is missing from one or more segments. | Structure | 6.9.3✓ error | 6.9.3✓ error | 6.9.3✓ error |
| `ST2067-3/DanglingEssenceDescriptor` | EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource. | Reference | 6.1.10.1✓ error | 6.1.10.1✓ error | 6.1.10.1✓ error |

## SMPTE ST 2067-5 — Essence Component

| Rule | Description | Category | 2013 | 2020 |
|------|-------------|----------|------|------|
| `ST2067-5/OperationalPatternQualifier` | Operational Pattern UL byte 15 qualifier bits 1-3 shall be 0 (internal essence, stream file, uni-track); with the ST 377-1 marker bit 0 set, the low nibble is 0x1. | Container | 5.2.1✓ error | 5.2.1✓ error |

## SMPTE ST 2067-9 — Sidecar Composition Map

| Rule | Description | Category | 2018 |
|------|-------------|----------|------|
| `ST2067-9/MalformedXml` | SidecarCompositionMap document is not well-formed XML (§6.1). | Reference | 6.1✓ critical |
| `ST2067-9/SidecarAssetReferencedByVirtualTrack` | A sidecar asset shall not be referenced by any Virtual Track in a CPL (§5). | Reference | 5✓ error |
| `ST2067-9/DuplicateAssetId` | Duplicate SidecarAsset Id within SidecarAssetList (§7.2.3). | Reference | 7.2.3✓ error |
| `ST2067-9/SignerWithoutSignature` | Signer element is present but the required Signature element is absent (§7.2.4). | Reference | 7.2.4✓ error |
| `ST2067-9/SignatureWithoutSigner` | Signature element is present but the required Signer element is absent (§7.2.5). | Reference | 7.2.5✓ error |
| `ST2067-9/SidecarAssetNotFound` | SidecarAsset Id is not present in the package AssetMap (§7.3.1). | Reference | 7.3.1✓ error |
| `ST2067-9/CplNotFound` | CPLId in AssociatedCPLList does not reference a known CPL in this package (§7.3.1.1). | Reference | 7.3.1.1✓ warning |
| `ST2067-9/DuplicateCplId` | Duplicate CPLId within a single AssociatedCPLList (§7.3.1.1). | Reference | 7.3.1.1✓ error |

## SMPTE ST 2067-21 — Application #2E

| Rule | Description | Category | 2014 | 2016 | 2020 | 2023 | 2026 |
|------|-------------|----------|------|------|------|------|------|
| `ST2067-21/ColorSystem` | Color system designator is not in the permitted set. | Video | 6.1✓ error | 6.1✓ error | 6.2✓ error | 6.2✓ error | 7.2✓ error |
| `ST2067-21/BlackRefLevel` | BlackRefLevel value is inconsistent with ComponentDepth. | Video | 6.1.4✓ error | 6.3✓ error | 6.4.3✓ error | 6.4.3✓ error | 7.4.3✓ error |
| `ST2067-21/WhiteRefLevel` | WhiteRefLevel value is inconsistent with ComponentDepth. | Video | 6.1.4✓ error | 6.3✓ error | 6.4.3✓ error | 6.4.3✓ error | 7.4.3✓ error |
| `ST2067-21/ColorRange` | ColorRange value is not in the permitted set. | Video | 6.1.4✓ error | 6.3✓ error | 6.4.3✓ error | 6.4.3✓ error | 7.4.3✓ error |
| `ST2067-21/ApplicationIdentification` | ApplicationIdentification is required for App2E compositions. | Metadata | 7✓ error | 7.1✓ error | 7.1✓ error | 7.1✓ error | 8.1✓ error |
| `ST2067-21/AppIdMismatch` | ApplicationIdentification in CPL ExtensionProperties does not include the expected App2E URI. | Metadata | 7✓ warning | 7.1✓ warning | 7.1✓ warning | 7.1✓ warning | 8.1✓ warning |
| `ST2067-21/ComponentRefValues` | Component max/min reference values are inconsistent with bit depth. | Video | —✓ | 6.2✓ error | 6.3.2✓ error | 6.3.2✓ error | 7.3.2✓ error |
| `ST2067-21/MaxCLLMaxFALL` | MaxCLL / MaxFALL HDR metadata is absent; recommended for HDR content. | Video | —✓ | 7.2✓ info | 7.5✓ info | 7.5✓ info | 8.3.3✓ info |
| `ST2067-21/FrameRate` | Frame rate is not in the permitted set for App2E. | Video | —✓ | —✓ | 5.2✓ error | 5.2✓ error | 6.2.1✓ error |
| `ST2067-21/Resolution` | Image resolution is not in the permitted set for App2E. | Video | —✓ | —✓ | 5.2✓ error | 5.2✓ error | 6.2.1✓ error |
| `ST2067-21/RequiredStoredWidth` | RGBA/CDCI descriptor is missing the required StoredWidth field. | Encoding | —✓ | —✓ | 6.2.1.1✓ error | 6.2.1.1✓ error | 7.2.1.2✓ error |
| `ST2067-21/RequiredStoredHeight` | RGBA/CDCI descriptor is missing the required StoredHeight field. | Encoding | —✓ | —✓ | 6.2.1.1✓ error | 6.2.1.1✓ error | 7.2.1.2✓ error |
| `ST2067-21/RequiredSampleRate` | RGBA/CDCI descriptor is missing the required SampleRate field. | Encoding | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/RequiredFrameLayout` | RGBA/CDCI descriptor is missing the required FrameLayout field. | Encoding | —✓ | —✓ | 6.2.1.3✓ error | 6.2.1.3✓ error | 7.2.1.4✓ error |
| `ST2067-21/RequiredColorPrimaries` | RGBA/CDCI descriptor is missing the required ColorPrimaries field. | Encoding | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/RequiredTransferCharacteristic` | RGBA/CDCI descriptor is missing the required TransferCharacteristic field. | Encoding | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/RequiredPictureCompression` | RGBA/CDCI descriptor is missing the required PictureCompression field. | Encoding | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/RequiredComponentDepth` | CDCI descriptor is missing the required ComponentDepth field. | Encoding | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/AlphaTransparency` | Alpha transparency mode is not permitted in App2E. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | —✓ |
| `ST2067-21/CodingEquationsMissing` | CodingEquations field is absent from the picture descriptor. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/ColorPrimariesMissing` | ColorPrimaries field is absent from the picture descriptor. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/FieldDominance` | FieldDominance value is not permitted for the declared FrameLayout. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/FrameLayout` | FrameLayout value is not in the permitted set for App2E. | Video | —✓ | —✓ | 6.2.1.3✓ error | 6.2.1.3✓ error | 7.2.1.4✓ error |
| `ST2067-21/FrameLayoutInterlaced` | Image frame structure (progressive or interlaced) does not fit the closest permitted column of the edition's image-characteristics table. | Video | —✓ | —✓ | 5.2✓ error | 5.2✓ error | 6.2.1✓ error |
| `ST2067-21/ImageAlignmentOffset` | ImageAlignmentOffset must be zero. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/ImageEndOffset` | ImageEndOffset must be zero. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/ImageStartOffset` | ImageStartOffset must be zero. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/SampledHeight` | SampledHeight must equal StoredHeight. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/SampledWidth` | SampledWidth must equal StoredWidth. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/SampledXOffset` | SampledXOffset must be zero. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/SampledYOffset` | SampledYOffset must be zero. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/StoredF2Offset` | StoredF2Offset must be zero. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/TransferCharacteristicMissing` | TransferCharacteristic field is absent from the picture descriptor. | Video | —✓ | —✓ | 6.2.1✓ error | 6.2.1✓ error | 7.2.1.1✓ error |
| `ST2067-21/TransferCharacteristicUnknown` | TransferCharacteristic UL is present but not a recognized value. | Video | — | — | 6.2.2✓ error | 6.2.2✓ error | 7.2.2✓ error |
| `ST2067-21/CodingEquationsUnknown` | CodingEquations UL is present but not a recognized value. | Video | — | — | 6.2.3✓ error | 6.2.3✓ error | 7.2.3✓ error |
| `ST2067-21/ColorPrimariesUnknown` | ColorPrimaries UL is present but not a recognized value. | Video | — | — | 6.2.4✓ error | 6.2.4✓ error | 7.2.4✓ error |
| `ST2067-21/J2KRequired` | Video essence is not JPEG 2000 encoded as required by App2E. | Encoding | —✓ | — | 6.2.5✓ error | 6.2.5✓ error | 7.2.5✓ error |
| `ST2067-21/AlphaMaxRef` | AlphaMaxRef value is not permitted. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/AlphaMinRef` | AlphaMinRef value is not permitted. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/ComponentMaxRef` | ComponentMaxRef value is not in the permitted range. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/ComponentMinRef` | ComponentMinRef value is not in the permitted range. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/Palette` | Palette is present; palette images are not permitted in App2E. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/PaletteLayout` | PaletteLayout is present; palette layout is not permitted in App2E. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/ScanningDirection` | ScanningDirection value is not in the permitted set. | Video | —✓ | —✓ | 6.3.1✓ error | 6.3.1✓ error | 7.3.1✓ error |
| `ST2067-21/AlphaSampleDepth` | AlphaSampleDepth value is not permitted. | Video | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/ColorSiting` | ColorSiting value is not in the permitted set. | Video | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/ComponentDepth` | ComponentDepth is missing, or the pixel bit depth is not permitted for the image characteristics. | Video | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/HorizontalSubsampling` | HorizontalSubsampling value is not in the permitted set. | Video | —✓ | —✓ | 6.4.2✓ error | 6.4.2✓ error | 7.4.2✓ error |
| `ST2067-21/PaddingBits` | PaddingBits must be zero. | Video | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/ReversedByteOrder` | ReversedByteOrder flag is set; byte reversal is not permitted. | Video | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/VerticalSubsampling` | VerticalSubsampling value is not in the permitted set. | Video | —✓ | —✓ | 6.4.1✓ error | 6.4.1✓ error | 7.4.1✓ error |
| `ST2067-21/CodingStyle` | JPEG 2000 codestream coding style is not compliant. | Encoding | —✓ | —✓ | 6.5.1✓ error | 6.5.1✓ error | 7.5.1✓ error |
| `ST2067-21/J2CLayout` | JPEG 2000 codestream layout does not meet App2E requirements. | Encoding | —✓ | —✓ | 6.5.1✓ error | 6.5.1✓ error | 7.5.1✓ error |
| `ST2067-21/Jpeg2000SubDescriptor` | JPEG2000SubDescriptor is absent or incomplete. | Encoding | —✓ | —✓ | 6.5.1✓ warning | 6.5.1✓ warning | 7.5.1✓ warning |
| `ST2067-21/J2KHtNotAllowed` | JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020. | Encoding | —✓ | — | 5.5.2✓ error | —✓ | —✓ |
| `ST2067-21/J2K4KResolution` | JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range. | Encoding | —✓ | — | 5.5.2✓ error | 5.5.2✓ error | 6.5.2✓ error |
| `ST2067-21/J2K2KResolution` | JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range. | Encoding | —✓ | — | 5.5.2✓ error | 5.5.2✓ error | 6.5.2✓ error |
| `ST2067-21/J2KBcpResolution` | JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range. | Encoding | —✓ | — | 5.5.2✓ error | 5.5.2✓ error | 6.5.2✓ error |
| `ST2067-21/J2KProfileNotAllowed` | The JPEG 2000 profile is not listed in the edition's JPEG 2000 profile table. | Encoding | —✓ | — | 5.5.2✓ error | 5.5.2✓ error | —✓ |
| `ST2067-21/HomogeneousImageEssence` | All image essence in a composition shall use the same color system. | Video | —✓ | —✓ | 7.2✓ error | 7.2✓ error | 8.2✓ error |
| `ST2067-21/SegmentDurationMultiple` | Segment duration must be an integer multiple of 5 edit units. | Timing | —✓ | —✓ | 7.4✓ error | 7.4✓ error | 8.3.2✓ error |
| `ST2067-21/J2KExtendedCapabilities` | J2KExtendedCapabilities shall be present when ISO/IEC 15444-15 (HTJ2K) coding is used, but it is missing. | Encoding | —✓ | —✓ | —✓ | 6.5.1✓ error | 7.5.1✓ error |
| `ST2067-21/HtIrvParameterBMax` | HTJ2K irreversible (APP2.HT.IRV) codestream: the MAGB_P set signalled in Ccap15 allows parameter B above 31. APP2.HT.REV is not checked: Table H.1 gives its B as a value calculated per Table H.2, not a limit (SMPTE st2067-21 issue 7, open for REV). | Encoding | —✓ | —✓ | —✓ | —✓ | H✓ error |
| `ST2067-21/HtIrvParameterBRecommended` | HTJ2K irreversible (APP2.HT.IRV) codestream: the MAGB_P set signalled in Ccap15 allows parameter B above the recommended value for its bit depth and multiple component transform. | Encoding | —✓ | —✓ | —✓ | —✓ | H✓ warning |
| `ST2067-21/J2K8KResolution` | JPEG 2000 IMF 8K Profile: stored resolution is outside the permitted range. | Encoding | —✓ | —✓ | —✓ | —✓ | 6.5.2✓ error |

## SMPTE ST 2067-201 — IAB Level 0 Plug-in

| Rule | Description | Category | 2019 | 2021 | 2026 |
|------|-------------|----------|------|------|------|
| `ST2067-201/CodecForbidden` | IABEssenceDescriptor: Codec item shall not be present (§5.9). | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/ElectrospatialFormulationInvalid` | IABEssenceDescriptor: ElectrospatialFormulation, if present, shall be 15 — multi-channel mode default (§5.9). | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/QuantizationBitsMissing` | IABEssenceDescriptor: QuantizationBits is missing; shall be 24. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/QuantizationBitsInvalid` | IABEssenceDescriptor: QuantizationBits shall be 24. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/ContainerFormatMissing` | IABEssenceDescriptor: ContainerFormat is missing. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/EssenceContainerInvalid` | IABEssenceDescriptor: ContainerFormat is not the required IAB container UL. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/AudioSamplingRateMissing` | IABEssenceDescriptor: AudioSampleRate shall be present (§5.9). | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/AudioSamplingRateInvalid` | IABEssenceDescriptor: AudioSampleRate shall match the bitstream SampleRate — 48 kHz or 96 kHz per ST 2098-2. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/SoundCompressionMissing` | IABEssenceDescriptor: SoundCompression is missing. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/SoundCompressionInvalid` | IABEssenceDescriptor: SoundCompression is not the required IAB compression UL. | Audio | 5.9✓ error | 5.9✓ error | 5.9✓ error |
| `ST2067-201/ChannelCountNotZero` | IABEssenceDescriptor: ChannelCount shall be the distinguished value 0 (2019 edition). | Audio | 5.9✓ error | —✓ | —✓ |
| `ST2067-201/SubDescriptorMissing` | IABEssenceDescriptor: IABSoundfieldLabelSubDescriptor shall be present. | Audio | 5.10.2✓ error | 5.10.2✓ error | 5.10.2✓ error |
| `ST2067-201/MCATagSymbolMissing` | IABSoundfieldLabelSubDescriptor: MCATagSymbol is missing; shall be "IAB". | Audio | 5.10.4✓ error | 5.10.4✓ error | 5.10.4✓ error |
| `ST2067-201/MCATagSymbolInvalid` | IABSoundfieldLabelSubDescriptor: MCATagSymbol shall be "IAB". | Audio | 5.10.4✓ error | 5.10.4✓ error | 5.10.4✓ error |
| `ST2067-201/MCATagNameMissing` | IABSoundfieldLabelSubDescriptor: MCATagName is missing; shall be "IAB". | Audio | 5.10.3✓ error | 5.10.3✓ error | 5.10.3✓ error |
| `ST2067-201/MCATagNameInvalid` | IABSoundfieldLabelSubDescriptor: MCATagName shall be "IAB". | Audio | 5.10.4✓ error | 5.10.4✓ error | 5.10.4✓ error |
| `ST2067-201/MCALabelDictionaryIDMissing` | IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is missing. | Audio | 5.10.4✓ error | 5.10.4✓ error | 5.10.4✓ error |
| `ST2067-201/MCALabelDictionaryIDInvalid` | IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is not the required IAB label UL. | Audio | 5.10.4✓ error | 5.10.4✓ error | 5.10.4✓ error |
| `ST2067-201/IABSequenceNoResources` | IABSequence shall contain at least one Resource (§6.2). | Audio | 6.2✓ error | 6.2✓ error | 6.2.1✓ error |
| `ST2067-201/IABSequenceSourceEncodingInvalid` | IABSequence Resource.SourceEncoding does not reference an IABEssenceDescriptor (§6.2). | Audio | 6.2✓ error | 6.2✓ error | 6.2.1✓ error |
| `ST2067-201/ForbiddenMCASubDescriptor` | IAB Track File shall not contain AudioChannelLabel, SoundfieldGroupLabel or GroupOfSoundfieldGroupsLabel SubDescriptors (§5.10.2). | Audio | 5.10.2✓ error | 5.10.2✓ error | 5.10.2✓ error |
| `ST2067-201/SubDescriptorDuplicate` | IAB Track File shall contain exactly one IABSoundfieldLabelSubDescriptor (§5.10.2). | Audio | 5.10.2✓ error | 5.10.2✓ error | 5.10.2✓ error |
| `ST2067-201/MCAChannelIDForbidden` | MCAChannelID shall not be present in the IAB Soundfield Label SubDescriptor (Annex C.2). | Audio | C.2✓ error | C.2✓ error | C.2✓ error |
| `ST2067-201/EditRateNotIntegerMultiple` | IAB Track File Edit Rate shall be an integer multiple of the Main Image Virtual Track Edit Rate (§6.2). | Audio | 6.2✓ error | 6.2✓ error | 6.2.1✓ error |
| `ST2067-201/IabChannelSubDescriptorRecommended` | Track File should carry an IABChannelSubDescriptor for each channel of each BedDefinition (§5.10.2; set defined in Annex E). | Audio | —✓ | —✓ | 5.10.2✓ warning |

## SMPTE ST 2067-202 — ISXD Plug-in

| Rule | Description | Category | 2023 |
|------|-------------|----------|------|
| `ST2067-202/NamespaceUriMissing` | ISXDDataEssenceDescriptor: NamespaceURI shall be present (§9.2 Table 5, Req). | Data | 9.2✓ error |
| `ST2067-202/ISXDSequenceNoResources` | ISXDSequence shall contain at least one Resource. | Data | 6✓ error |
| `ST2067-202/ISXDSequenceSourceEncodingInvalid` | ISXDSequence Resource.SourceEncoding does not reference an ISXDDataEssenceDescriptor. | Data | 6✓ error |
| `ST2067-202/NamespaceUriMismatch` | All ISXD Track Files referenced by an ISXD Virtual Track shall have an identical NamespaceURI value (§6). | Data | 6✓ error |
| `ST2067-202/EditRateMismatch` | The Edit Rate of an ISXD Virtual Track shall be equal to the Edit Rate of the Main Image Virtual Track (§6). | Data | 6✓ error |
| `ST2067-202/ISXDVirtualTrackMissing` | A Composition that references an ISXD Track File shall contain one or more ISXD Virtual Tracks (§6). | Data | 6✓ error |
| `ST2067-202/DataEssenceCodingMissing` | ISXDDataEssenceDescriptor: DataEssenceCoding shall be present (§9.3). | Data | 9.3✓ error |
| `ST2067-202/DataEssenceCodingInvalid` | ISXDDataEssenceDescriptor: DataEssenceCoding shall be the UTF-8 Text Data Essence Coding UL (§9.3 Table 6). | Data | 9.3✓ error |

## XML Schema validation

| Rule | Description | Category | CPL-2013 | CPL-2016 | PKL-2016 | DCI-PKL-2007 | SCM-2018 | OPL-2014 | custom | unvendored |
|------|-------------|----------|------|------|------|------|------|------|------|------|
| `XSD/ElementMissing` | A required element is absent (XSD minOccurs violated). | Schema | · error | · error | · error | · error | · error | · error | · error | — |
| `XSD/UnexpectedElement` | An element appears outside its declared content model. | Schema | · error | · error | · error | · error | · error | · error | · error | — |
| `XSD/PatternInvalid` | A value violates an xs:pattern facet on its type. | Schema | · error | · error | · error | · error | · error | · error | · error | — |
| `XSD/TypeInvalid` | A value violates its declared XSD type (built-in or restriction). | Schema | · error | · error | · error | · error | · error | · error | · error | — |
| `XSD/SchemaConstraintFailed` | An XSD constraint failed; see message for details. | Schema | · error | · error | · error | · error | · error | · error | · error | — |
| `XSD/ValidationAborted` | XSD validation aborted: the schema or the document failed to parse, or the schema could not be compiled. | Schema | · critical | · critical | · critical | · critical | · critical | · critical | · critical | — |
| `XSD/NotValidated` | XSD validation was skipped: no vendored schema covers this document's namespace. | Schema | — | — | — | — | — | — | — | · info |

## imferno engine

| Rule | Description | Category | Clause |
|------|-------------|----------|------|
| `IMFERNO/EditionAssumed` | The edition of a spec applied to a document (the package, a CPL, the Asset Map) could not be determined: its namespace is shared by several editions and nothing in it tells them apart, or the evidence names no single one of them. The newest candidate was applied; the message names the namespace seen and the edition used. | Structure | Package info |
| `IMFERNO/EditionConflict` | The namespace of a document allows only one edition of a spec, but other evidence in it (e.g. the ApplicationIdentification) names another edition. The namespace's edition was applied; the message names both editions and the evidence. | Structure | Package warning |
| `IMFERNO/EditionPinned` | The edition of a spec applied to a document (the package, a CPL, an MXF track file) was set by the caller: pinned by an option (coreSpec / --core-spec, cplSpec / --cpl-spec, iabSpec / --iab-spec), or following from a pinned ST 2067-2 edition through its dated references. The message names the edition applied, the pin it comes from, and what the document itself declares and would give on its own. Reported whenever a pin was applied, also when it agrees with the document. | Structure | Package info |
| `IMFERNO/DeliveryNotChecked` | The package follows ST 2067-2:2013, which defines no delivery requirements (§7: an IMP is a Packing List and the assets it references): the delivery rules of later editions (Asset Map ingest, ASSETMAP.xml name, ST 429-9 Annex A paths, VOLINDEX) were not checked. Packing List Size and Hash are checked under ST 429-8. | Structure | Package info |
| `IMFERNO/OpenFailed` | MXF file could not be opened for reading. | Container | Mxf critical |
| `IMFERNO/PartitionPackParseFailed` | MXF header partition pack failed to parse. | Container | Mxf critical |
| `IMFERNO/RegXmlConversionFailed` | MXF header metadata could not be converted to RegXML. | Container | Mxf warning |
| `IMFERNO/MxfHeaderUnreadable` | MXF file could not be parsed; it may be truncated or corrupt. | Asset | Mxf warning |
| `IMFERNO/EssenceContainersDetected` | Informational trace of how many essence containers the MXF declares. | Container | Mxf info |
| `IMFERNO/IoError` | An I/O error prevented the asset from being read. | Asset | General error |
| `IMFERNO/UnreferencedAsset` | Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM. | Structure | Package info |
| `IMFERNO/UnlistedEssence` | File is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader. | Structure | Package info |
| `IMFERNO/ParseError` | IMF package failed to parse due to a structural error. | Structure | Package critical |
| `IMFERNO/PklParseError` | A Packing List referenced by the AssetMap could not be parsed. | Structure | Package error |
| `IMFERNO/XmlAssetParseError` | An XML asset could not be parsed as CPL, OPL, or SCM. | Structure | Package warning |
| `IMFERNO/XmlReadError` | An XML file could not be read from disk. | Structure | Package warning |
| `IMFERNO/ReadDirError` | Could not scan the package directory. | Structure | Package info |
| `IMFERNO/DirEntryError` | Could not read a directory entry while scanning for unlisted essences. | Structure | Package info |
| `IMFERNO/PathTraversal` | An asset chunk path attempts to escape the package root directory (path traversal). | Structure | Package error |
| `IMFERNO/ApplicationProfileNotValidated` | Application-profile validation was not performed for the CPL: ApplicationIdentification is absent or lists URIs imferno has no validator for. | Structure | Package warning |
| `IMFERNO/RuleNotInEdition` | Internal: a check reported a rule against an edition whose table row says the edition does not contain it. The finding was dropped; please report this as an imferno bug. | Structure | Engine warning |

