---
title: Validation Codes
description: Complete reference of all validation codes across every supported SMPTE spec.
---

Every validation issue emitted by imferno carries a code like `ST2067-2:2020:8.3/FileNotFound`. Use these codes to [configure rule severity](/guide/config/).

## ST 429-9 — Volume Index

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST429-9:2014:7/VolindexMissing` | No volume-index document found in the package root. | INFO | Structure |
| `ST429-9:2014:7/MalformedXml` | The VOLINDEX.xml document is not well-formed XML. | ERROR | Structure |

## ST 377-1 — MXF File Format

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST377-1:2011:5/NotMxf` | File is not a valid MXF container. | WARNING | Asset |
| `ST377-1:2011:5/ParseError` | MXF file could not be parsed; it may be truncated or corrupt. | WARNING | Asset |
| `ST377-1:2011:11/NoEssenceContainers` | MXF file contains no essence containers. | WARNING | Encoding |
| `ST377-1:2011:7/OP1a` | MXF operational pattern must be OP1a for IMF packages. | ERROR | Encoding |

## ST 2067-2 — Core Constraints & Packing List

### Package-level (2020)

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-2:2020:7/AssetMap` | AssetMap document is invalid or cannot be parsed. | CRITICAL | Structure |
| `ST2067-2:2020:7/MalformedXml` | The ASSETMAP.xml document is not well-formed XML. | ERROR | Structure |
| `ST2067-2:2020:9/MalformedXml` | A Packing List document is not well-formed XML. | ERROR | Structure |
| `ST2067-2:2020:7/NoCpls` | No CPL assets found in the AssetMap. | CRITICAL | Structure |
| `ST2067-2:2020:8.3/SizeMismatch` | Declared file size does not match the on-disk size. | ERROR | Asset |
| `ST2067-2:2020:8.3/FileNotFound` | A referenced asset file is not present at the declared path. | ERROR | Asset |
| `ST2067-2:2020:8.3/ChecksumMismatch` | File hash does not match the declared SHA-1/SHA-256 checksum. | ERROR | Asset |
| `ST2067-2:2020:7/UnresolvedUuid` | UUID referenced in the CPL does not resolve to a known asset. | ERROR | Reference |
| `ST2067-2:2020:7/DuplicateUuid` | Two or more assets within the package share the same UUID. | ERROR | Reference |
| `IMF:General/IoError` | An I/O error prevented the asset from being read. | ERROR | Asset |
| `ST2067-2:2020:6.4.2/EssenceDescriptorList` | EssenceDescriptorList element is required per ST 2067-2:2020 §6.4.2. | ERROR | Structure |

### Core Constraints — 2013

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-2:2013:XSD/ResourceList-Empty` | A Sequence has an empty ResourceList. | ERROR | Structure |
| `ST2067-2:2013:XSD/ContentTitle` | ContentTitle shall not be empty. | ERROR | Metadata |
| `ST2067-2:2013:XSD/TotalRunningTime-Format` | TotalRunningTime does not match required format HH:MM:SS. | ERROR | Structure |
| `ST2067-2:2013:XSD/SegmentList` | SegmentList shall contain at least one Segment. | CRITICAL | Structure |
| `ST2067-2:2013:XSD/Segment` | A Segment contains no sequences. | ERROR | Structure |
| `ST2067-2:2013:XSD-88/EditRate` | CPL EditRate is required (XSD schema §88). | ERROR | Structure |
| `ST2067-2:2013:XSD-66/IssueDate` | IssueDate shall not be empty. | ERROR | Metadata |
| `ST2067-2:2013:XSD-66/IssueDate-Format` | IssueDate is not a valid xs:dateTime format. | WARNING | Metadata |
| `ST2067-2:2013:XSD-121-127/CompositionTimecode-DropFrame` | CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2013:XSD-121-127/CompositionTimecode-Rate` | CompositionTimecode.TimecodeRate is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2013:XSD-121-127/CompositionTimecode-StartAddress` | CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2013:XSD-121-127/CompositionTimecode-Rate-Zero` | CompositionTimecode.TimecodeRate shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2013:XSD-121-127/CompositionTimecode-StartAddress-Format` | TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF. | ERROR | Timing |
| `ST2067-2:2013:XSD-121-127/CompositionTimecode-RateMismatch` | CompositionTimecode.TimecodeRate does not match the CPL EditRate. | WARNING | Metadata |
| `ST2067-2:2013:XSD/LocaleList-NonEmpty` | LocaleList shall contain at least one Locale. | ERROR | Structure |
| `ST2067-2:2013:6.1/UniqueSegmentId` | Duplicate Segment Id within the CPL. | ERROR | Structure |
| `ST2067-2:2013:6.1/UniqueEssenceDescriptorId` | Duplicate EssenceDescriptor Id within the CPL. | ERROR | Structure |
| `ST2067-2:2013:6.1/UniqueResourceId` | Duplicate Resource Id within the CPL. | ERROR | Structure |
| `ST2067-2:2013:6.10/IntrinsicDuration` | IntrinsicDuration shall be greater than 0. | ERROR | Timing |
| `ST2067-2:2013:6.10/EntryPoint` | EntryPoint shall be less than IntrinsicDuration. | ERROR | Timing |
| `ST2067-2:2013:6.10/SourceDuration` | EntryPoint + SourceDuration exceeds IntrinsicDuration. | ERROR | Timing |
| `ST2067-2:2013:6.10/ResourceDuration` | SourceDuration shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2013:6.10/RepeatCount` | RepeatCount shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2013:6.10/TrackFileId` | A non-marker resource is missing a TrackFileId. | ERROR | Reference |
| `ST2067-2:2013:6.9/VirtualTrackContinuity` | A virtual track is missing from one or more segments. | ERROR | Structure |
| `ST2067-2:2013:6.9.3/VirtualTrackEditRate` | All resources in a virtual track shall have the same edit rate. | ERROR | Timing |
| `ST2067-2:2013:10/TimedText-SampleRate` | DCTimedTextDescriptor SampleRate is missing. | WARNING | Subtitle |
| `ST2067-2:2013:10/TimedText-EmptyLanguageTag` | Empty language tag in RFC5646LanguageTagList. | WARNING | Subtitle |
| `ST2067-2:2013:10/TimedText-MalformedLanguageTag` | Language tag does not start with an ASCII letter (RFC 5646 primary subtag). | WARNING | Subtitle |
| `ST2067-2:2013:ST377-4/AudioSampleRate` | WAVEPCMDescriptor has no AudioSampleRate or SampleRate. | WARNING | Audio |
| `ST2067-2:2013:ST377-4/ChannelCount` | WAVEPCMDescriptor ChannelCount is zero or missing. | WARNING | Audio |
| `ST2067-2:2013:ST377-4/MCASubDescriptors` | WAVEPCMDescriptor has no MCA SubDescriptors. | WARNING | Audio |
| `ST2067-2:2013:ST377-4/SoundfieldGroup` | WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor. | WARNING | Audio |
| `ST2067-2:2013:ST377-4/MCATagSymbol` | SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol. | WARNING | Audio |
| `ST2067-2:2013:ST377-4/SoundfieldChannelCount` | Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount. | ERROR | Audio |
| `ST2067-2:2013:8/DigitalSignature` | Digital signature validation (ST 2067-2 §8) is not currently performed. | INFO | Security |
| `ST2067-2:2013:6.4.2/DanglingEssenceDescriptor` | EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource. | ERROR | Reference |
| `ST2067-2:2013:6.4.2/EssenceDescriptorList` | EssenceDescriptorList is required per ST 2067-2 §6.4.2. | ERROR | Structure |

### Core Constraints — 2016

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-2:2016:XSD/ResourceList-Empty` | A Sequence has an empty ResourceList. | ERROR | Structure |
| `ST2067-2:2016:XSD/ContentTitle` | ContentTitle shall not be empty. | ERROR | Metadata |
| `ST2067-2:2016:XSD/TotalRunningTime-Format` | TotalRunningTime does not match required format HH:MM:SS. | ERROR | Structure |
| `ST2067-2:2016:XSD/SegmentList` | SegmentList shall contain at least one Segment. | CRITICAL | Structure |
| `ST2067-2:2016:XSD/Segment` | A Segment contains no sequences. | ERROR | Structure |
| `ST2067-2:2016:XSD-88/EditRate` | CPL EditRate is required (XSD schema §88). | ERROR | Structure |
| `ST2067-2:2016:XSD-66/IssueDate` | IssueDate shall not be empty. | ERROR | Metadata |
| `ST2067-2:2016:XSD-66/IssueDate-Format` | IssueDate is not a valid xs:dateTime format. | WARNING | Metadata |
| `ST2067-2:2016:XSD-121-127/CompositionTimecode-DropFrame` | CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2016:XSD-121-127/CompositionTimecode-Rate` | CompositionTimecode.TimecodeRate is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2016:XSD-121-127/CompositionTimecode-StartAddress` | CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2016:XSD-121-127/CompositionTimecode-Rate-Zero` | CompositionTimecode.TimecodeRate shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2016:XSD-121-127/CompositionTimecode-StartAddress-Format` | TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF. | ERROR | Timing |
| `ST2067-2:2016:XSD-121-127/CompositionTimecode-RateMismatch` | CompositionTimecode.TimecodeRate does not match the CPL EditRate. | WARNING | Metadata |
| `ST2067-2:2016:XSD/LocaleList-NonEmpty` | LocaleList shall contain at least one Locale. | ERROR | Structure |
| `ST2067-2:2016:6.1/UniqueSegmentId` | Duplicate Segment Id within the CPL. | ERROR | Structure |
| `ST2067-2:2016:6.1/UniqueEssenceDescriptorId` | Duplicate EssenceDescriptor Id within the CPL. | ERROR | Structure |
| `ST2067-2:2016:6.1/UniqueResourceId` | Duplicate Resource Id within the CPL. | ERROR | Structure |
| `ST2067-2:2016:6.10/IntrinsicDuration` | IntrinsicDuration shall be greater than 0. | ERROR | Timing |
| `ST2067-2:2016:6.10/EntryPoint` | EntryPoint shall be less than IntrinsicDuration. | ERROR | Timing |
| `ST2067-2:2016:6.10/SourceDuration` | EntryPoint + SourceDuration exceeds IntrinsicDuration. | ERROR | Timing |
| `ST2067-2:2016:6.10/ResourceDuration` | SourceDuration shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2016:6.10/RepeatCount` | RepeatCount shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2016:6.10/TrackFileId` | A non-marker resource is missing a TrackFileId. | ERROR | Reference |
| `ST2067-2:2016:6.9/VirtualTrackContinuity` | A virtual track is missing from one or more segments. | ERROR | Structure |
| `ST2067-2:2016:6.9.3/VirtualTrackEditRate` | All resources in a virtual track shall have the same edit rate. | ERROR | Timing |
| `ST2067-2:2016:10/TimedText-SampleRate` | DCTimedTextDescriptor SampleRate is missing. | WARNING | Subtitle |
| `ST2067-2:2016:10/TimedText-EmptyLanguageTag` | Empty language tag in RFC5646LanguageTagList. | WARNING | Subtitle |
| `ST2067-2:2016:10/TimedText-MalformedLanguageTag` | Language tag does not start with an ASCII letter (RFC 5646 primary subtag). | WARNING | Subtitle |
| `ST2067-2:2016:ST377-4/AudioSampleRate` | WAVEPCMDescriptor has no AudioSampleRate or SampleRate. | WARNING | Audio |
| `ST2067-2:2016:ST377-4/ChannelCount` | WAVEPCMDescriptor ChannelCount is zero or missing. | WARNING | Audio |
| `ST2067-2:2016:ST377-4/MCASubDescriptors` | WAVEPCMDescriptor has no MCA SubDescriptors. | WARNING | Audio |
| `ST2067-2:2016:ST377-4/SoundfieldGroup` | WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor. | WARNING | Audio |
| `ST2067-2:2016:ST377-4/MCATagSymbol` | SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol. | WARNING | Audio |
| `ST2067-2:2016:ST377-4/SoundfieldChannelCount` | Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount. | ERROR | Audio |
| `ST2067-2:2016:8/DigitalSignature` | Digital signature validation (ST 2067-2 §8) is not currently performed. | INFO | Security |
| `ST2067-2:2016:6.4.2/DanglingEssenceDescriptor` | EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource. | ERROR | Reference |
| `ST2067-2:2016:6.4.2/EssenceDescriptorList` | EssenceDescriptorList is required per ST 2067-2 §6.4.2. | ERROR | Structure |

### Core Constraints — 2020

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-2:2020:XSD/ResourceList-Empty` | A Sequence has an empty ResourceList. | ERROR | Structure |
| `ST2067-2:2020:XSD/ContentTitle` | ContentTitle shall not be empty. | ERROR | Metadata |
| `ST2067-2:2020:XSD/TotalRunningTime-Format` | TotalRunningTime does not match required format HH:MM:SS. | ERROR | Structure |
| `ST2067-2:2020:XSD/SegmentList` | SegmentList shall contain at least one Segment. | CRITICAL | Structure |
| `ST2067-2:2020:XSD/Segment` | A Segment contains no sequences. | ERROR | Structure |
| `ST2067-2:2020:XSD-88/EditRate` | CPL EditRate is required (XSD schema §88). | ERROR | Structure |
| `ST2067-2:2020:XSD-66/IssueDate` | IssueDate shall not be empty. | ERROR | Metadata |
| `ST2067-2:2020:XSD-66/IssueDate-Format` | IssueDate is not a valid xs:dateTime format. | WARNING | Metadata |
| `ST2067-2:2020:XSD-121-127/CompositionTimecode-DropFrame` | CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2020:XSD-121-127/CompositionTimecode-Rate` | CompositionTimecode.TimecodeRate is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2020:XSD-121-127/CompositionTimecode-StartAddress` | CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present. | ERROR | Timing |
| `ST2067-2:2020:XSD-121-127/CompositionTimecode-Rate-Zero` | CompositionTimecode.TimecodeRate shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2020:XSD-121-127/CompositionTimecode-StartAddress-Format` | TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF. | ERROR | Timing |
| `ST2067-2:2020:XSD-121-127/CompositionTimecode-RateMismatch` | CompositionTimecode.TimecodeRate does not match the CPL EditRate. | WARNING | Metadata |
| `ST2067-2:2020:XSD/LocaleList-NonEmpty` | LocaleList shall contain at least one Locale. | ERROR | Structure |
| `ST2067-2:2020:6.1/UniqueSegmentId` | Duplicate Segment Id within the CPL. | ERROR | Structure |
| `ST2067-2:2020:6.1/UniqueEssenceDescriptorId` | Duplicate EssenceDescriptor Id within the CPL. | ERROR | Structure |
| `ST2067-2:2020:6.1/UniqueResourceId` | Duplicate Resource Id within the CPL. | ERROR | Structure |
| `ST2067-2:2020:6.10/IntrinsicDuration` | IntrinsicDuration shall be greater than 0. | ERROR | Timing |
| `ST2067-2:2020:6.10/EntryPoint` | EntryPoint shall be less than IntrinsicDuration. | ERROR | Timing |
| `ST2067-2:2020:6.10/SourceDuration` | EntryPoint + SourceDuration exceeds IntrinsicDuration. | ERROR | Timing |
| `ST2067-2:2020:6.10/ResourceDuration` | SourceDuration shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2020:6.10/RepeatCount` | RepeatCount shall be a positive integer. | ERROR | Timing |
| `ST2067-2:2020:6.10/TrackFileId` | A non-marker resource is missing a TrackFileId. | ERROR | Reference |
| `ST2067-2:2020:6.9/VirtualTrackContinuity` | A virtual track is missing from one or more segments. | ERROR | Structure |
| `ST2067-2:2020:6.9.3/VirtualTrackEditRate` | All resources in a virtual track shall have the same edit rate. | ERROR | Timing |
| `ST2067-2:2020:10/TimedText-SampleRate` | DCTimedTextDescriptor SampleRate is missing. | WARNING | Subtitle |
| `ST2067-2:2020:10/TimedText-EmptyLanguageTag` | Empty language tag in RFC5646LanguageTagList. | WARNING | Subtitle |
| `ST2067-2:2020:10/TimedText-MalformedLanguageTag` | Language tag does not start with an ASCII letter (RFC 5646 primary subtag). | WARNING | Subtitle |
| `ST2067-2:2020:ST377-4/AudioSampleRate` | WAVEPCMDescriptor has no AudioSampleRate or SampleRate. | WARNING | Audio |
| `ST2067-2:2020:ST377-4/ChannelCount` | WAVEPCMDescriptor ChannelCount is zero or missing. | WARNING | Audio |
| `ST2067-2:2020:ST377-4/MCASubDescriptors` | WAVEPCMDescriptor has no MCA SubDescriptors. | WARNING | Audio |
| `ST2067-2:2020:ST377-4/SoundfieldGroup` | WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor. | WARNING | Audio |
| `ST2067-2:2020:ST377-4/MCATagSymbol` | SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol. | WARNING | Audio |
| `ST2067-2:2020:ST377-4/SoundfieldChannelCount` | Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount. | ERROR | Audio |
| `ST2067-2:2020:8/DigitalSignature` | Digital signature validation (ST 2067-2 §8) is not currently performed. | INFO | Security |
| `ST2067-2:2020:6.4.2/DanglingEssenceDescriptor` | EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource. | ERROR | Reference |
| `ST2067-2:2020:6.4.2/EssenceDescriptorList` | EssenceDescriptorList is required per ST 2067-2 §6.4.2. | ERROR | Structure |

## ST 2067-3 — Composition Playlist

### 2013

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-3:2013:5.5.1.2/ContentKindUnknown` | ContentKind uses an unrecognized value under the SMPTE scope. | WARNING | Metadata |
| `ST2067-3:2013:6.4.2/SourceEncodingNoEssenceDescriptorList` | SourceEncoding present but EssenceDescriptorList absent. | ERROR | Reference |
| `ST2067-3:2013:6.4.2/SourceEncodingUnresolved` | SourceEncoding does not match any EssenceDescriptor Id. | ERROR | Reference |
| `ST2067-3:2013:6.4.2/EssenceDescriptorListEmpty` | EssenceDescriptorList present but contains no descriptors. | ERROR | Structure |
| `ST2067-3:2013:6.11/ContentVersionListEmpty` | ContentVersionList present but empty. | ERROR | Structure |
| `ST2067-3:2013:6.11/ContentVersionIdInvalid` | ContentVersion/Id is empty (shall be a URI). | ERROR | Metadata |
| `ST2067-3:2013:6.11/ContentVersionLabelTextMissing` | ContentVersion/LabelText is absent. | WARNING | Metadata |
| `ST2067-3:2013:6.12/LocaleLanguageTagInvalid` | Locale language tag does not conform to RFC 5646. | WARNING | Metadata |
| `ST2067-3:2013:7.3/TrackIdNotUnique` | TrackId is not unique within a segment. | ERROR | Structure |
| `ST2067-3:2013:7.4/MarkerOffsetOutOfRange` | Marker offset exceeds resource effective duration. | ERROR | Timing |
| `ST2067-3:2013:7.4/MarkerLabelUnknown` | Marker label is not a recognized SMPTE standard value. | WARNING | Metadata |
| `ST2067-3:2013:7.2.2/SegmentDuration` | All virtual tracks in a segment must span the same number of edit units. | ERROR | Timing |
| `ST2067-3:2013:6.1.9/ContentVersionIdDuplicate` | No two ContentVersion elements shall have identical Id values. | ERROR | Structure |
| `ST2067-3:2013:7.3/SegmentDurationIntegerEditUnits` | Sequence duration shall be an integer number of Composition Edit Units. | ERROR | Timing |

### 2016

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-3:2016:5.5.1.2/ContentKindUnknown` | ContentKind uses an unrecognized value under the SMPTE scope. | WARNING | Metadata |
| `ST2067-3:2016:6.4.2/SourceEncodingNoEssenceDescriptorList` | SourceEncoding present but EssenceDescriptorList absent. | ERROR | Reference |
| `ST2067-3:2016:6.4.2/SourceEncodingUnresolved` | SourceEncoding does not match any EssenceDescriptor Id. | ERROR | Reference |
| `ST2067-3:2016:6.4.2/EssenceDescriptorListEmpty` | EssenceDescriptorList present but contains no descriptors. | ERROR | Structure |
| `ST2067-3:2016:6.11/ContentVersionListEmpty` | ContentVersionList present but empty. | ERROR | Structure |
| `ST2067-3:2016:6.11/ContentVersionIdInvalid` | ContentVersion/Id is empty (shall be a URI). | ERROR | Metadata |
| `ST2067-3:2016:6.11/ContentVersionLabelTextMissing` | ContentVersion/LabelText is absent. | WARNING | Metadata |
| `ST2067-3:2016:6.12/LocaleLanguageTagInvalid` | Locale language tag does not conform to RFC 5646. | WARNING | Metadata |
| `ST2067-3:2016:7.3/TrackIdNotUnique` | TrackId is not unique within a segment. | ERROR | Structure |
| `ST2067-3:2016:7.4/MarkerOffsetOutOfRange` | Marker offset exceeds resource effective duration. | ERROR | Timing |
| `ST2067-3:2016:7.4/MarkerLabelUnknown` | Marker label is not a recognized SMPTE standard value. | WARNING | Metadata |
| `ST2067-3:2016:7.2.2/SegmentDuration` | All virtual tracks in a segment must span the same number of edit units. | ERROR | Timing |
| `ST2067-3:2016:6.1.9/ContentVersionIdDuplicate` | No two ContentVersion elements shall have identical Id values. | ERROR | Structure |
| `ST2067-3:2016:7.3/SegmentDurationIntegerEditUnits` | Sequence duration shall be an integer number of Composition Edit Units. | ERROR | Timing |

### 2020

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-3:2020:5.5.1.2/ContentKindUnknown` | ContentKind uses an unrecognized value under the SMPTE scope. | WARNING | Metadata |
| `ST2067-3:2020:6.4.2/SourceEncodingNoEssenceDescriptorList` | SourceEncoding present but EssenceDescriptorList absent. | ERROR | Reference |
| `ST2067-3:2020:6.4.2/SourceEncodingUnresolved` | SourceEncoding does not match any EssenceDescriptor Id. | ERROR | Reference |
| `ST2067-3:2020:6.4.2/EssenceDescriptorListEmpty` | EssenceDescriptorList present but contains no descriptors. | ERROR | Structure |
| `ST2067-3:2020:6.11/ContentVersionListEmpty` | ContentVersionList present but empty. | ERROR | Structure |
| `ST2067-3:2020:6.11/ContentVersionIdInvalid` | ContentVersion/Id is empty (shall be a URI). | ERROR | Metadata |
| `ST2067-3:2020:6.11/ContentVersionLabelTextMissing` | ContentVersion/LabelText is absent. | WARNING | Metadata |
| `ST2067-3:2020:6.12/LocaleLanguageTagInvalid` | Locale language tag does not conform to RFC 5646. | WARNING | Metadata |
| `ST2067-3:2020:7.3/TrackIdNotUnique` | TrackId is not unique within a segment. | ERROR | Structure |
| `ST2067-3:2020:7.4/MarkerOffsetOutOfRange` | Marker offset exceeds resource effective duration. | ERROR | Timing |
| `ST2067-3:2020:7.4/MarkerLabelUnknown` | Marker label is not a recognized SMPTE standard value. | WARNING | Metadata |
| `ST2067-3:2020:7.2.2/SegmentDuration` | All virtual tracks in a segment must span the same number of edit units. | ERROR | Timing |
| `ST2067-3:2020:6.1.9/ContentVersionIdDuplicate` | No two ContentVersion elements shall have identical Id values. | ERROR | Structure |
| `ST2067-3:2020:7.3/SegmentDurationIntegerEditUnits` | Sequence duration shall be an integer number of Composition Edit Units. | ERROR | Timing |

## ST 2067-9 — Sidecar Composition Map

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-9:2018:6.1/MalformedXml` | SidecarCompositionMap document is not well-formed XML (§6.1). | CRITICAL | Reference |
| `ST2067-9:2018:5/SidecarAssetReferencedByVirtualTrack` | A sidecar asset shall not be referenced by any Virtual Track in a CPL (§5). | ERROR | Reference |
| `ST2067-9:2018:7.2.3/DuplicateAssetId` | Duplicate SidecarAsset Id within SidecarAssetList (§7.2.3). | ERROR | Reference |
| `ST2067-9:2018:7.2.4/SignerWithoutSignature` | Signer element is present but the required Signature element is absent (§7.2.4). | ERROR | Reference |
| `ST2067-9:2018:7.2.5/SignatureWithoutSigner` | Signature element is present but the required Signer element is absent (§7.2.5). | ERROR | Reference |
| `ST2067-9:2018:7.3.1/SidecarAssetNotFound` | SidecarAsset Id is not present in the package AssetMap (§7.3.1). | ERROR | Reference |
| `ST2067-9:2018:7.3.1.1/CplNotFound` | CPLId in AssociatedCPLList does not reference a known CPL in this package (§7.3.1.1). | ERROR | Reference |
| `ST2067-9:2018:7.3.1.1/DuplicateCplId` | Duplicate CPLId within a single AssociatedCPLList (§7.3.1.1). | ERROR | Reference |

## ST 2067-21 — Application #2E

### 2020

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-21:2020:7.1/AppIdMismatch` | Application identifier in CPL ExtensionProperties does not match the expected App2E URI. | WARNING | Metadata |

### 2023

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-21:2023:5.2/FrameRate` | Frame rate is not in the permitted set for App2E. | ERROR | Video |
| `ST2067-21:2023:5.2/Resolution` | Image resolution is not in the permitted set for App2E. | ERROR | Video |
| `ST2067-21:2023:5.3/EmptyLanguageTag` | Locale language tag is empty. | ERROR | Metadata |
| `ST2067-21:2023:5.3/MalformedLanguageTag` | Locale language tag is not a valid BCP-47 subtag. | ERROR | Metadata |
| `ST2067-21:2023:5.3/RegionCode` | Region subtag in a language tag is not valid. | ERROR | Metadata |
| `ST2067-21:2023:6.2/ColorSystem` | Color system designator is not in the permitted set. | ERROR | Video |
| `ST2067-21:2023:6.2/Required-StoredWidth` | RGBA/CDCI descriptor is missing the required StoredWidth field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-StoredHeight` | RGBA/CDCI descriptor is missing the required StoredHeight field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-SampleRate` | RGBA/CDCI descriptor is missing the required SampleRate field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-FrameLayout` | RGBA/CDCI descriptor is missing the required FrameLayout field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-ColorPrimaries` | RGBA/CDCI descriptor is missing the required ColorPrimaries field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-TransferCharacteristic` | RGBA/CDCI descriptor is missing the required TransferCharacteristic field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-PictureCompression` | RGBA/CDCI descriptor is missing the required PictureCompression field. | ERROR | Encoding |
| `ST2067-21:2023:6.2/Required-ComponentDepth` | CDCI descriptor is missing the required ComponentDepth field. | ERROR | Encoding |
| `ST2067-21:2023:6.5/Required-ChannelCount` | WavePCM descriptor is missing the required ChannelCount field. | ERROR | Audio |
| `ST2067-21:2023:6.5/Required-QuantizationBits` | WavePCM descriptor is missing the required QuantizationBits field. | ERROR | Audio |
| `ST2067-21:2023:6.2.1/AlphaTransparency` | Alpha transparency mode is not permitted in App2E. | ERROR | Video |
| `ST2067-21:2023:6.2.1/CodingEquations` | CodingEquations field is absent from the picture descriptor (Table 8). | ERROR | Video |
| `ST2067-21:2023:6.2.1/ColorPrimaries` | ColorPrimaries field is absent from the picture descriptor (Table 8). | ERROR | Video |
| `ST2067-21:2023:6.2.1/FieldDominance` | FieldDominance value is not permitted for the declared FrameLayout. | ERROR | Video |
| `ST2067-21:2023:6.2.1/FrameLayout` | FrameLayout value is not in the permitted set for App2E. | ERROR | Video |
| `ST2067-21:2023:6.2.1/FrameLayoutInterlaced` | FrameLayout declares interlaced content, which is not permitted in App2E. | ERROR | Video |
| `ST2067-21:2023:6.2.1/ImageAlignmentOffset` | ImageAlignmentOffset must be zero. | ERROR | Video |
| `ST2067-21:2023:6.2.1/ImageEndOffset` | ImageEndOffset must be zero. | ERROR | Video |
| `ST2067-21:2023:6.2.1/ImageStartOffset` | ImageStartOffset must be zero. | ERROR | Video |
| `ST2067-21:2023:6.2.1/SampledHeight` | SampledHeight must equal StoredHeight. | ERROR | Video |
| `ST2067-21:2023:6.2.1/SampledWidth` | SampledWidth must equal StoredWidth. | ERROR | Video |
| `ST2067-21:2023:6.2.1/SampledXOffset` | SampledXOffset must be zero. | ERROR | Video |
| `ST2067-21:2023:6.2.1/SampledYOffset` | SampledYOffset must be zero. | ERROR | Video |
| `ST2067-21:2023:6.2.1/StoredF2Offset` | StoredF2Offset must be zero. | ERROR | Video |
| `ST2067-21:2023:6.2.1/TransferCharacteristic` | TransferCharacteristic field is absent from the picture descriptor (Table 8). | ERROR | Video |
| `ST2067-21:2023:6.2.2/TransferCharacteristic` | TransferCharacteristic UL is present but not a recognized value. | ERROR | Video |
| `ST2067-21:2023:6.2.3/CodingEquations` | CodingEquations UL is present but not a recognized value. | ERROR | Video |
| `ST2067-21:2023:6.2.4/ColorPrimaries` | ColorPrimaries UL is present but not a recognized value. | ERROR | Video |
| `ST2067-21:2023:6.2.5/J2KRequired` | Video essence is not JPEG 2000 encoded as required by App2E. | ERROR | Encoding |
| `ST2067-21:2023:6.3/AlphaMaxRef` | AlphaMaxRef value is not permitted. | ERROR | Video |
| `ST2067-21:2023:6.3/AlphaMinRef` | AlphaMinRef value is not permitted. | ERROR | Video |
| `ST2067-21:2023:6.3/ComponentMaxRef` | ComponentMaxRef value is not in the permitted range. | ERROR | Video |
| `ST2067-21:2023:6.3/ComponentMinRef` | ComponentMinRef value is not in the permitted range. | ERROR | Video |
| `ST2067-21:2023:6.3/Palette` | Palette is present; palette images are not permitted in App2E. | ERROR | Video |
| `ST2067-21:2023:6.3/PaletteLayout` | PaletteLayout is present; palette layout is not permitted in App2E. | ERROR | Video |
| `ST2067-21:2023:6.3/ScanningDirection` | ScanningDirection value is not in the permitted set. | ERROR | Video |
| `ST2067-21:2023:6.3.2/ComponentRefValues` | Component max/min reference values are inconsistent with bit depth. | ERROR | Video |
| `ST2067-21:2023:6.4/AlphaSampleDepth` | AlphaSampleDepth value is not permitted. | ERROR | Video |
| `ST2067-21:2023:6.4/ColorSiting` | ColorSiting value is not in the permitted set. | ERROR | Video |
| `ST2067-21:2023:6.4/ComponentDepth` | ComponentDepth value is not in the permitted set (8 / 10 / 12 / 16). | ERROR | Video |
| `ST2067-21:2023:6.4/HorizontalSubsampling` | HorizontalSubsampling value is not in the permitted set. | ERROR | Video |
| `ST2067-21:2023:6.4/PaddingBits` | PaddingBits must be zero. | ERROR | Video |
| `ST2067-21:2023:6.4/ReversedByteOrder` | ReversedByteOrder flag is set; byte reversal is not permitted. | ERROR | Video |
| `ST2067-21:2023:6.4/VerticalSubsampling` | VerticalSubsampling value is not in the permitted set. | ERROR | Video |
| `ST2067-21:2023:6.4.3/BlackRefLevel` | BlackRefLevel value is inconsistent with ComponentDepth. | ERROR | Video |
| `ST2067-21:2023:6.4.3/ColorRange` | ColorRange value is not in the permitted set. | ERROR | Video |
| `ST2067-21:2023:6.4.3/WhiteRefLevel` | WhiteRefLevel value is inconsistent with ComponentDepth. | ERROR | Video |
| `ST2067-21:2023:6.5/AudioSampleRate` | Audio sample rate must be 48 000 Hz. | ERROR | Audio |
| `ST2067-21:2023:6.5/QuantizationBits` | QuantizationBits must be 16 or 24. | ERROR | Audio |
| `ST2067-21:2023:6.5.2/CodingStyle` | JPEG 2000 codestream coding style is not compliant. | ERROR | Encoding |
| `ST2067-21:2023:6.5.2/J2CLayout` | JPEG 2000 codestream layout does not meet App2E requirements. | ERROR | Encoding |
| `ST2067-21:2023:6.5.2/J2KExtendedCapabilities` | JPEG 2000 extended capabilities are declared but not permitted. | ERROR | Encoding |
| `ST2067-21:2023:6.5.2/JPEG2000SubDescriptor` | JPEG2000SubDescriptor is absent or incomplete. | WARNING | Encoding |
| `ST2067-21:2023:6.2.5/J2K-HT-Not-Allowed` | JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020. | ERROR | Encoding |
| `ST2067-21:2023:6.2.5/J2K-4K-Resolution` | JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range. | ERROR | Encoding |
| `ST2067-21:2023:6.2.5/J2K-2K-Resolution` | JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range. | ERROR | Encoding |
| `ST2067-21:2023:6.2.5/J2K-BCP-Resolution` | JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range. | ERROR | Encoding |
| `ST2067-21:2023:7.1/ApplicationIdentification` | ApplicationIdentification is required for App2E compositions. | ERROR | Metadata |
| `ST2067-21:2023:7.1/ContentMaturityRating-Agency` | ContentMaturityRating Agency is empty. | ERROR | Metadata |
| `ST2067-21:2023:7.1/ContentMaturityRating-Agency-URI` | ContentMaturityRating Agency is not a valid xs:anyURI. | ERROR | Metadata |
| `ST2067-21:2023:7.2/HomogeneousImageEssence` | All image essence in a composition shall use the same color system. | ERROR | Video |
| `ST2067-21:2023:7.1/AppIdMismatch` | Application identifier in CPL ExtensionProperties does not match the expected App2E URI. | WARNING | Metadata |
| `ST2067-21:2023:7.4/SegmentDurationMultiple` | Segment duration must be an integer multiple of 5 edit units. | ERROR | Timing |
| `ST2067-21:2023:7.5/MaxCLLMaxFALL` | MaxCLL / MaxFALL HDR metadata is absent; recommended for HDR content. | INFO | Video |

### 2025

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-21:2025:5.6/FNTimedText` | Timed text track designated as Forced Narrative (FN) does not comply with §5.6. | ERROR | Subtitle |
| `ST2067-21:2025:5.6/HICTimedText` | Timed text track designated as HI-Caption (HIC) does not comply with §5.6. | ERROR | Subtitle |

## ST 2067-201 — IAB Plug-in

### 2019

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-201:2019:5.9/CodecForbidden` | IABEssenceDescriptor: Codec item shall not be present (§5.9). | ERROR | Audio |
| `ST2067-201:2019:5.9/ElectrospatialFormulationForbidden` | IABEssenceDescriptor: ElectrospatialFormulation shall not be present (§5.9). | ERROR | Audio |
| `ST2067-201:2019:5.9/QuantizationBitsMissing` | IABEssenceDescriptor: QuantizationBits is missing; shall be 24. | WARNING | Audio |
| `ST2067-201:2019:5.9/QuantizationBitsInvalid` | IABEssenceDescriptor: QuantizationBits shall be 24. | ERROR | Audio |
| `ST2067-201:2019:5.3/ContainerFormatMissing` | IABEssenceDescriptor: ContainerFormat is missing. | WARNING | Audio |
| `ST2067-201:2019:5.3/EssenceContainerInvalid` | IABEssenceDescriptor: ContainerFormat is not the required IAB container UL. | ERROR | Audio |
| `ST2067-201:2019:5.9/AudioSamplingRateMissing` | IABEssenceDescriptor: AudioSampleRate is missing; shall be 48000/1. | WARNING | Audio |
| `ST2067-201:2019:5.9/AudioSamplingRateInvalid` | IABEssenceDescriptor: AudioSampleRate shall be 48000/1. | ERROR | Audio |
| `ST2067-201:2019:5.9/SoundCompressionMissing` | IABEssenceDescriptor: SoundCompression is missing. | WARNING | Audio |
| `ST2067-201:2019:5.9/SoundCompressionInvalid` | IABEssenceDescriptor: SoundCompression is not the required IAB compression UL. | ERROR | Audio |
| `ST2067-201:2019:5.9/ChannelCountNotZero` | IABEssenceDescriptor: ChannelCount shall be the distinguished value 0 (2019 edition). | ERROR | Audio |
| `ST2067-201:2019:5.9/SubDescriptorMissing` | IABEssenceDescriptor: IABSoundfieldLabelSubDescriptor shall be present. | ERROR | Audio |
| `ST2067-201:2019:5.9/MCATagSymbolMissing` | IABSoundfieldLabelSubDescriptor: MCATagSymbol is missing; shall be "IAB". | ERROR | Audio |
| `ST2067-201:2019:5.9/MCATagSymbolInvalid` | IABSoundfieldLabelSubDescriptor: MCATagSymbol shall be "IAB". | ERROR | Audio |
| `ST2067-201:2019:5.9/MCATagNameMissing` | IABSoundfieldLabelSubDescriptor: MCATagName is missing; shall be "IAB". | ERROR | Audio |
| `ST2067-201:2019:5.9/MCATagNameInvalid` | IABSoundfieldLabelSubDescriptor: MCATagName shall be "IAB". | ERROR | Audio |
| `ST2067-201:2019:5.9/MCALabelDictionaryIDMissing` | IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is missing. | ERROR | Audio |
| `ST2067-201:2019:5.9/MCALabelDictionaryIDInvalid` | IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is not the required IAB label UL. | ERROR | Audio |
| `ST2067-201:2019:6.2/MainAudioMissing` | Segment has IABSequence but no MainAudioSequence (§6.2). | ERROR | Audio |
| `ST2067-201:2019:6.2/IABSequenceNoResources` | IABSequence shall contain at least one Resource (§6.2). | ERROR | Audio |
| `ST2067-201:2019:6.2/IABSequenceSourceEncodingInvalid` | IABSequence Resource.SourceEncoding does not reference an IABEssenceDescriptor (§6.2). | ERROR | Audio |

### 2021

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-201:2021:5.9/CodecForbidden` | IABEssenceDescriptor: Codec item shall not be present (§5.9). | ERROR | Audio |
| `ST2067-201:2021:5.9/ElectrospatialFormulationForbidden` | IABEssenceDescriptor: ElectrospatialFormulation shall not be present (§5.9). | ERROR | Audio |
| `ST2067-201:2021:5.9/QuantizationBitsMissing` | IABEssenceDescriptor: QuantizationBits is missing; shall be 24. | WARNING | Audio |
| `ST2067-201:2021:5.9/QuantizationBitsInvalid` | IABEssenceDescriptor: QuantizationBits shall be 24. | ERROR | Audio |
| `ST2067-201:2021:5.3/ContainerFormatMissing` | IABEssenceDescriptor: ContainerFormat is missing. | WARNING | Audio |
| `ST2067-201:2021:5.3/EssenceContainerInvalid` | IABEssenceDescriptor: ContainerFormat is not the required IAB container UL. | ERROR | Audio |
| `ST2067-201:2021:5.9/AudioSamplingRateMissing` | IABEssenceDescriptor: AudioSampleRate is missing; shall be 48000/1. | WARNING | Audio |
| `ST2067-201:2021:5.9/AudioSamplingRateInvalid` | IABEssenceDescriptor: AudioSampleRate shall be 48000/1. | ERROR | Audio |
| `ST2067-201:2021:5.9/SoundCompressionMissing` | IABEssenceDescriptor: SoundCompression is missing. | WARNING | Audio |
| `ST2067-201:2021:5.9/SoundCompressionInvalid` | IABEssenceDescriptor: SoundCompression is not the required IAB compression UL. | ERROR | Audio |
| `ST2067-201:2021:5.9/ChannelCountNotZero` | IABEssenceDescriptor: ChannelCount shall be the distinguished value 0 (2019 edition). | ERROR | Audio |
| `ST2067-201:2021:5.9/SubDescriptorMissing` | IABEssenceDescriptor: IABSoundfieldLabelSubDescriptor shall be present. | ERROR | Audio |
| `ST2067-201:2021:5.9/MCATagSymbolMissing` | IABSoundfieldLabelSubDescriptor: MCATagSymbol is missing; shall be "IAB". | ERROR | Audio |
| `ST2067-201:2021:5.9/MCATagSymbolInvalid` | IABSoundfieldLabelSubDescriptor: MCATagSymbol shall be "IAB". | ERROR | Audio |
| `ST2067-201:2021:5.9/MCATagNameMissing` | IABSoundfieldLabelSubDescriptor: MCATagName is missing; shall be "IAB". | ERROR | Audio |
| `ST2067-201:2021:5.9/MCATagNameInvalid` | IABSoundfieldLabelSubDescriptor: MCATagName shall be "IAB". | ERROR | Audio |
| `ST2067-201:2021:5.9/MCALabelDictionaryIDMissing` | IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is missing. | ERROR | Audio |
| `ST2067-201:2021:5.9/MCALabelDictionaryIDInvalid` | IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is not the required IAB label UL. | ERROR | Audio |
| `ST2067-201:2021:6.2/MainAudioMissing` | Segment has IABSequence but no MainAudioSequence (§6.2). | ERROR | Audio |
| `ST2067-201:2021:6.2/IABSequenceNoResources` | IABSequence shall contain at least one Resource (§6.2). | ERROR | Audio |
| `ST2067-201:2021:6.2/IABSequenceSourceEncodingInvalid` | IABSequence Resource.SourceEncoding does not reference an IABEssenceDescriptor (§6.2). | ERROR | Audio |

## ST 2067-202 — ISXD Plug-in

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `ST2067-202:2022:5/SubDescriptorMissing` | ISXDDataEssenceDescriptor: ContainerConstraintsSubDescriptor shall be present. | ERROR | Audio |
| `ST2067-202:2022:5/NamespaceUriMissing` | ISXDDataEssenceDescriptor: NamespaceURI is absent. | WARNING | Audio |
| `ST2067-202:2022:6/ISXDSequenceNoResources` | ISXDSequence shall contain at least one Resource. | ERROR | Audio |
| `ST2067-202:2022:6/ISXDSequenceSourceEncodingInvalid` | ISXDSequence Resource.SourceEncoding does not reference an ISXDDataEssenceDescriptor. | ERROR | Audio |
| `ST2067-202:2022:6/NamespaceUriMismatch` | Resources in the same ISXDSequence reference descriptors with inconsistent NamespaceURI values. | ERROR | Audio |

## imferno

Codes emitted by imferno's package-level logic for conditions that don't map to a specific SMPTE spec clause.

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `IMFERNO:Package/UnreferencedAsset` | Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM. | INFO | Structure |
| `IMFERNO:Package/UnlistedEssence` | MXF file is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader. | WARNING | Structure |
| `IMFERNO:Package/ParseError` | IMF package failed to parse due to a structural error. | CRITICAL | Structure |
| `IMFERNO:Package/PklParseError` | A Packing List referenced by the AssetMap could not be parsed. | ERROR | Structure |
| `IMFERNO:Package/XmlAssetParseError` | An XML asset could not be parsed as CPL, OPL, or SCM. | WARNING | Structure |
| `IMFERNO:Package/XmlReadError` | An XML file could not be read from disk. | WARNING | Structure |
| `IMFERNO:Package/ReadDirError` | Could not scan the package directory. | INFO | Structure |
| `IMFERNO:Package/DirEntryError` | Could not read a directory entry while scanning for unlisted essences. | INFO | Structure |
| `IMFERNO:Package/PathTraversal` | An asset chunk path attempts to escape the package root directory (path traversal). | ERROR | Structure |
