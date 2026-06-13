---
spec: ST 2067-2
edition: 2013
title: IMF Core Constraints
xsd: specs/imf-core-constraints-20130620.xsd
xsd_sha256: 5e87522b8ac7de5bb35917843ad0c90cb20e4b8ae611020f8f5c494dbccf1764
xsd_lines: 35
namespace: http://www.smpte-ra.org/schemas/2067-2/2013
prose_url: https://pub.smpte.org/doc/st2067-2/20130829-pub/st2067-2-2013.pdf
prose_sha256: <verified by fetch.py — see frontmatter on regen>
prose_pages: 31
catalogue_files:
  - crates/imferno-core/src/assetmap/codes.rs
imports:
  - http://www.smpte-ra.org/schemas/2067-3/2013 (CPL types: SequenceType, BaseResourceType, TrackFileResourceType, UUIDType via dcml)
---

## Summary
- 11 XSD constructs inventoried
- ~50 prose-only constraints (Core Constraints is mostly prose; XSD is a thin layer adding extension elements)
- 0 conflicts
- ST 2067-2 layers on top of ST 2067-3 — the XSD imports the 2067-3 namespace for `cpl:SequenceType` and `cpl:BaseResourceType`
- The XSD only declares **extension elements** (Sequences, ApplicationIdentification, StereoImageTrackFileResourceType); the bulk of Core Constraints is prose specifying Track File formats, codec choices, audio/video constraints, segment/composition rules

## A. XSD construct inventory

| XSD line(s) | Construct | Prose § | Status | Notes |
|---|---|---|---|---|
| 1-4 | schema decl + import 2067-3 | §4.1 | matches | XML Schema and Namespace |
| 6 | `<xs:element name="TimedTextResourceID" type="dcml:UUIDType"/>` | §5.4.x (Timed Text Descriptor extension) | matches | UUID identifier for timed text resources |
| 7-16 | `ApplicationIdentification` xs:simpleType (xs:list of xs:anyURI, minLength=1) | §6.1 (Table 10) | matches | "shall include a single instance of the ApplicationIdentification element specified in Table 10"; "Each unique item ... shall identify an Application to which a Composition conforms" |
| 17 | `MainImageSequence` SequenceType | §6.3.1 (Table 12) | matches | "Main Image Virtual Track shall consist of one or more MainImageSequence elements" |
| 18 | `MainAudioSequence` SequenceType | §6.3.2 (Table 13) | matches | "Each Audio Virtual Track shall consist of one or more MainAudioSequence elements" |
| 19 | `SubtitlesSequence` SequenceType | §6.3.3 (Table 14) | matches | Data Essence Virtual Track sequence kind |
| 20 | `HearingImpairedCaptionsSequence` SequenceType | §6.3.3 (Table 14) | matches | Data Essence Virtual Track sequence kind |
| 21 | `VisuallyImpairedTextSequence` SequenceType | §6.3.3 (Table 14) | matches | Data Essence Virtual Track sequence kind |
| 22 | `CommentarySequence` SequenceType | §6.3.3 (Table 14) | matches | Data Essence Virtual Track sequence kind |
| 23 | `KaraokeSequence` SequenceType | §6.3.3 (Table 14) | matches | Data Essence Virtual Track sequence kind |
| 24 | `AncillaryDataSequence` SequenceType | §6.3.4 (Table 16) | matches | "Each Ancillary Data Virtual Track shall consist of one or more instances" |
| 25-34 | `StereoImageTrackFileResourceType` (extends BaseResourceType with LeftEye/RightEye TrackFileResourceType) | §6.3.1 / Annex D | matches | "if the underlying image essence consists of a sequence of pairs of image frames for stereoscopic viewing" |

## B. Prose constraints not in XSD

| Prose § | Constraint | Category | Notes |
|---|---|---|---|
| §4.2 | UUIDs "shall be generated as specified in IETF RFC 4122" | external standard | RFC 4122 conformance |
| §4.3 | XML Character Encoding constraint | encoding | XSD does not enforce |
| §5.1.1 | Track Files "shall be wrapped using SMPTE ST 379-2 (MXF, OP-1a)" | external essence format | MXF, not XML |
| §5.1.2 | Shim Parameters constraint | MXF parameter | not XML |
| §5.1.3 | Body Partition layout | MXF structural | not XML |
| §5.1.4 | Index Tables | MXF | not XML |
| §5.1.5 | Track File Identification — Package UID material number → TrackFileId | cross-MXF/CPL invariant | computed identity |
| §5.1.6 | MIME Type "application/mxf" | transport metadata | not in XML body |
| §5.2 | Image Track Files: format, alternative center cut, active area | MXF essence | not XML |
| §5.3.2 | Audio Essence: sampling rate, quantization | value-set | not XML; cross-file consistency |
| §5.3.4 | Wave Audio Essence Descriptor | MXF | not XML |
| §5.3.5 | Additional Generic Sound Essence Descriptor Items | MXF | not XML |
| §5.3.6 | Multichannel Audio Labeling (MCA / SoundfieldGroup / GroupOfSoundfieldGroups) | MXF MCA labels | not XML |
| §5.4.1 | Data Essence Track Files | external | TT format |
| §5.4.3 | TT essence "shall conform to SMPTE ST 2052-1" | external standard | not XML |
| §5.4.4 | RFC 5646 Language Tag List | external value-set | BCP-47 language tags |
| §5.4.5 | Image Resources URI uniqueness within DE Track File | cross-element uniqueness | "no two image resources referenced in a single Data Essence Track File shall have identical URI" |
| §5.4.5 | Image format "shall be PNG"; MIME "image/png"; AncillaryResourceID "shall be a Type 5 UUID" | computed identity | algorithm-specified UUID derivation |
| §5.4.6 | Font Resources: ISO/IEC 14496-18; MIME "application/x-font-opentype"; Type 5 UUID | computed identity | algorithm-specified UUID derivation |
| §5.5 | Ancillary Data Track Files "shall conform to SMPTE ST 436-1" | external standard | not XML |
| §6.1 | CPL ExtensionProperties "shall include a single instance of the ApplicationIdentification element" | cross-element conditional | XSD declares the element; prose requires it be present in CPL/ExtensionProperties |
| §6.1 | "Each Application shall define a value ... uniquely associated with its normative provisions" | semantic | external Application catalogue |
| §6.2.1 | Audio: "Quantization" and "Sampling Rate" "shall remain constant" within a composition | cross-file consistency | computed invariant across all audio track files in CPL |
| §6.3.1 | "A Composition shall contain exactly one Main Image Virtual Track" | conditional cardinality | XSD doesn't restrict to "exactly one" Image VT |
| §6.3.1 | If stereoscopic: MainImageSequence Resources shall be StereoImageTrackFileResourceType; if monoscopic: TrackFileResourceType | conditional type | mode-dependent type selection |
| §6.3.1 | "Edit Rate of the Resource elements shall be equal to the image frame rate of the underlying essence" | cross-MXF/CPL | invariant computed from MXF |
| §6.3.2 | "A Composition shall contain one or more Audio Virtual Tracks" | conditional cardinality | min cardinality on Virtual Tracks |
| §6.3.2 | "All Audio Essence Track Files referenced by a given Virtual Track shall have identical sets of GroupOfSoundfieldGroupsLabelSubDescriptors, SoundfieldGroupLabelSubDescriptors and AudioChannelLabelSubDescriptors instances" | cross-file MCA consistency | cross-MXF MCA label structural equivalence |
| §6.3.2 | "Edit Rate of the Resource elements shall be equal to the audio sampling rate of the underlying essence" | cross-MXF/CPL | invariant computed from MXF |
| §6.3.3 | Data Essence Virtual Tracks: zero or more | conditional cardinality | structural |
| §6.3.3 | Data Essence Resource: native start point = 0 in DE timeline; native duration ≥ time after which no DE active | semantic invariant | cross-MXF/CPL |
| §6.3.4 | Ancillary Data Virtual Tracks: zero or more | conditional cardinality | structural |
| §6.4 | "Composition Edit Rate shall be equal to the edit rate of the image essence referenced by the Main Image Virtual Track" | cross-VT consistency | computed invariant |
| §6.5 | TrackFileId UUID "shall be equal to the material number part of the Package UID of the Top-level File Package" | cross-MXF/CPL identity | computed equality |
| §6.6 | "duration of a Segment shall be greater than or equal to the duration of one image essence frame" | cross-VT bound | computed comparison |
| §6.7 | "A Composition Playlist instance shall contain at least one ContentVersion element" | within-CPL min cardinality | tightens 2067-3 (which makes it minOccurs=0 implicitly via list optional) |
| §6.8 | EssenceDescriptor "shall be mapped to a single EssenceDescriptor element ... using a single RegXML fragment as specified in SMPTE ST 2001-1" | computed mapping | RegXML serialization of MXF descriptor |
| §6.8 | Descriptors/Sub Descriptors to map: Wave Audio, MCA Label, Timed Text, Timed Text Resource | external mapping spec | catalogue of MXF types to RegXML |
| §6.9 | Signature: KeyInfo present with full cert chain | conditional structural | within Signature element |
| §6.9 | Signature: enveloped (Object absent, Reference URI=empty string) | crypto invariant | XMLDSig structural |
| §6.9 | Signature: DigestMethod algorithm "shall be set to ... sha256"; SignatureMethod "shall be set to ... rsa-sha256"; CanonicalizationMethod "shall be set to ... xml-c14n-20010315"; Transform "shall be set to ... enveloped-signature" | crypto algorithm value-set | URI value enumeration |
| §6.9 | X.509 cert chain "shall be carried in the KeyInfo element as a sequence of X509Data elements"; "each ... shall contain one X509IssuerSerial element and one X509Certificate element"; Distinguished Name "shall be compliant with RFC 2253" | crypto sub-structure | conditional XML structure under Signer |
| §6.10 | "digital certificate used shall conform to SMPTE ST 430-2" | external standard | cert profile |
| §7.1 | IMP "shall consist of one Packing List, as specified in SMPTE ST 429-8" | package-level | structural, package-scope |
| §7.2 | Asset.Id "shall uniquely identify the asset and shall be unique in the Packing List instance" | within-PKL uniqueness | could be xs:unique in PKL XSD |
| §7.2 | Asset.Id derivation per asset type | computed identity | algorithm per asset type |
| §7.3 | IMP "Digital Signature and Certs" | structural | per §6.9/6.10 |
| §7.4 | IMP Group ID | identity | package metadata |
| §7.5 / §7.6 | Complete vs Partial IMP | structural classification | package-level |

## C. Conflicts (prose vs XSD)

_None found._ Core Constraints intentionally leaves most of the
heavy lifting to prose since the constraints involve MXF essence,
cross-file consistency, and computed identities that XSD can't
express.

## D. Gaps

| Item | Reason |
|---|---|
| ApplicationIdentification value-set | XSD: list of xs:anyURI. Prose: each item "shall identify an Application" but the set of valid Application URIs is defined out-of-band by per-Application specs (e.g. ST 2067-21 = App #2E URI). XSD permissive; prose semantically tighter without enumeration. |
| §5.1 / §5.2 / §5.3 MXF Track File constraints | XSD doesn't apply (MXF essence is binary, not XML) — these constraints are addressed by separate non-XML validation in the engine |
| Cross-file consistency rules (§6.2.1, §6.3.x, §6.4) | XSD has no cross-file view; these are validator-level invariants |
| §6.5 Package UID → TrackFileId computed equality | requires reading MXF Package UID and comparing to CPL TrackFileId; cross-MXF/CPL |
| §6.7 "at least one ContentVersion" | ST 2067-2 tightens ST 2067-3 (which has minOccurs=0 on the list); XSD here doesn't express the tightening — only prose |

## Catalogue coverage cross-check

The catalogue uses one `St2067_2_2020` enum (used by all 3 editions
via dispatch) plus the macro-generated `St2067_2_2013_Core` /
`St2067_2_2016_Core` / `St2067_2_2020_Core` for the CPL-overlap
constraints. Mapped against Section B rows:

| Section B row | Implemented? | Catalogue code |
|---|---|---|
| §6.1 ApplicationIdentification presence in CPL | partial | implied by `St2067_21_2020::AppIdMismatch` (App #2E specific); generic Application ID presence not separately checked |
| §6.2.1 Audio quantization/sample-rate constancy | yes | `CoreConstraintsCode::AudioSampleRate`, `ChannelCount` |
| §6.3.1 Exactly one Main Image Virtual Track | gap | not enforced |
| §6.3.1 Stereoscopic Resource type selection | gap | not enforced |
| §6.3.2 At least one Audio Virtual Track | gap | not enforced |
| §6.3.x All Resource Edit Rate = essence rate | gap | not enforced (cross-MXF/CPL) |
| §6.4 Composition Edit Rate = main image rate | partial | `St2067_2_2020::EssenceDescriptorListEmpty` and related descriptor checks |
| §6.5 TrackFileId = MXF Package UID material | gap | not enforced (cross-MXF/CPL) |
| §6.6 Segment duration ≥ one image frame | gap | covered structurally by `CoreConstraintsCode::SegmentDuration` for integer-EU multiples |
| §6.7 ≥1 ContentVersion | partial | `CoreConstraintsCode::ContentVersionListEmpty` checks list non-empty if present, but not the cross-CPL min |
| §6.8 EssenceDescriptor RegXML mapping | n/a | descriptive only |
| §6.9 Signature crypto constraints | gap | signature crypto not verified (engine reports `St2067_2_2020::DigitalSignature` as documented gap) |
| §6.9 X.509 sub-structure | gap | not verified |
| §6.10 ST 430-2 cert profile | gap | not verified |
| §7.x IMP rules | partial | covered by `St2067_2_2020::*` AssetMap/PKL parse and hash codes |

**Coverage summary**: ST 2067-2 prose is dominated by MXF essence
and crypto rules that aren't XML-validatable. The engine implements
the high-leverage structural checks (AssetMap parse, PKL parse,
hash, hash-on-disk, missing files) but the deep MXF-side checks
(§6.5 UID equality, §6.3.x edit-rate equality) are gaps and would
require parsing MXF essence to verify.
