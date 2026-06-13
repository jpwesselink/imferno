---
spec: ST 2067-3
edition: 2013
title: Composition Playlist
xsd: specs/imf-cpl.xsd
xsd_sha256: 36c2aa65d1995df2b3cc2d1644bc74e908f2d5946d359fd204f40f779f56ceba
xsd_lines: 221
namespace: http://www.smpte-ra.org/schemas/2067-3/2013
prose_url: https://pub.smpte.org/doc/st2067-3/20130221-pub/st2067-3-2013.pdf
prose_sha256: 2a63250015a221535d90325bf611c9307dd29e1d3439cb9be9f5175c0628fda1
prose_pages: 34
catalogue_files:
  - crates/imferno-core/src/cpl/codes.rs
---

## Summary
- 53 XSD constructs inventoried
- 37 prose-only constraints classified
- 0 conflicts
- §5.1 declares the schema normative and gives prose precedence in conflict
- §6 intro explicitly delegates cardinality + defaults to the schema only

## A. XSD construct inventory

| XSD line(s) | Construct | Prose § | Status | Notes |
|---|---|---|---|---|
| 1-5 | schema targetNamespace + dcml/dsig imports | §5.1 (Table 1) | matches | Table 1 reproduces the schema root verbatim |
| 8 | `<xs:element name="CompositionPlaylist">` (root) | §5 / §6.1 | matches | "consists of a single CompositionPlaylist element" |
| 9-67 | `CompositionPlaylistType` complexType | §6.1 (Table 2) | matches | Table 2 reproduces this complexType verbatim |
| 10-66 | `<xs:sequence>` over 18 children | §6 intro | schema-only | "cardinality and default values of elements are specified in the schema only" |
| 11 | `Id` UUIDType required | §6.1.1 | matches | "shall uniquely identify the Composition Playlist instance" |
| 12 | `Annotation` UserTextType minOccurs=0 | §6.1.2 | matches | "free-form, human-readable annotation describing the composition" |
| 13 | `IssueDate` xs:dateTime required | §6.1.3 | matches | "shall indicate the time and date" |
| 14 | `Issuer` UserTextType minOccurs=0 | §6.1.4 | matches | "identifies the entity that created the Composition Playlist" |
| 15 | `Creator` UserTextType minOccurs=0 | §6.1.5 | matches | "identifies the device or software program used to create" |
| 16 | `ContentOriginator` UserTextType minOccurs=0 | §6.1.6 | matches | "identifies the originator of the content" |
| 17 | `ContentTitle` UserTextType required | §6.1.7 | matches | "shall contain a human-readable title for the composition" |
| 18 | `ContentKind` ContentKindType minOccurs=0 | §6.1.8 | matches | "shall be human-readable and indicate the kind of work" |
| 19-25 | `ContentVersionList` minOccurs=0, ContentVersion maxOccurs=unbounded | §6.1.9 | matches | structural |
| 26-33 | `EssenceDescriptorList` minOccurs=0, EssenceDescriptor maxOccurs=unbounded | §6.1.10, §6.1.10.1 | matches | structural |
| 34 | `CompositionTimecode` CompositionTimecodeType minOccurs=0 | §6.1.11 | matches | "provides the information necessary to associate a canonical synthetic timecode" |
| 35 | `EditRate` RationalType required | §6.1.12 | matches | "shall define the Composition Edit Rate" |
| 36-42 | `TotalRunningTime` regex `[0-9][0-9]:[0-5][0-9]:[0-5][0-9]` minOccurs=0 | §6.1.13 | matches | "as hours:minutes:seconds" |
| 43-49 | `LocaleList` minOccurs=0, Locale maxOccurs=unbounded | §6.1.14 | matches | structural |
| 50-56 | `ExtensionProperties` xs:any namespace=##other | §6.1.15, §5.4 | matches | extension via different namespace per §5.4 versioning rule |
| 57-63 | `SegmentList` required, Segment maxOccurs=unbounded | §6.1.16 | matches | "shall contain an ordered list of Segment elements" |
| 64 | `Signer` ds:KeyInfoType minOccurs=0 | §6.1.17 | matches | "shall uniquely identify the entity that digitally signed" |
| 65 | `Signature` ds:Signature ref minOccurs=0 | §6.1.18 | matches | "shall contain a digital signature authenticating the Composition Playlist" |
| 68-74 | `CompositionTimecodeType` | §6.2 (Table 4) | matches | Table 4 reproduces this complexType |
| 70 | `TimecodeDropFrame` xs:boolean required | §6.2.1.1 | matches | "shall indicate if the timecode is drop frame... or non-drop frame" |
| 71 | `TimecodeRate` xs:positiveInteger required | §6.2.1.2 | under-constrained | XSD allows any positive int. Prose: "shall specify the nearest integer frames per second rate of the timecode, e.g. 24, 30, 25" — illustrative, no exhaustive set |
| 72 | `TimecodeStartAddress` TimecodeType required | §6.2.1.3 | matches | "shall specify the value of the timecode at the beginning of the Composition" |
| 75-81 | `TimecodeType` xs:pattern (HH:MM:SS:FF with multi-separator) | §6.3 (Table 5) | matches | regex reproduced in Table 5; field semantics enumerated in §6.3 bulleted list |
| 82-89 | `ContentKindType` simpleContent with `scope` attribute default | §6.1.8 / §6.4 (Table 6) | matches | scope default URI is the normative trigger for Table 3 value-set per §6.1.8 |
| 90-116 | `LocaleType` | §6.5 (Table 7) | matches | structural |
| 92 | `Annotation` UserTextType minOccurs=0 | §6.5.1 | matches | "free-form, human-readable annotation describing the Locale" |
| 93-99 | `LanguageList` minOccurs=0, Language xs:string maxOccurs=unbounded | §6.5.2 | under-constrained | XSD: xs:string. Prose: "shall be a Language Tag as specified in [RFC 5646]" |
| 100-106 | `RegionList` minOccurs=0, Region xs:string maxOccurs=unbounded | §6.5.3 | under-constrained | XSD: xs:string. Prose: "shall be a valid region subtag of the Language Subtag Registry" |
| 107-114 | `ContentMaturityRatingList` minOccurs=0, ContentMaturityRating maxOccurs=unbounded | §6.5.4 | matches | structural |
| 117-132 | `ContentMaturityRatingType` | §6.6 (Table 8) | matches | structural |
| 119 | `Agency` xs:anyURI required | §6.6.1 | matches | "shall uniquely identify the agency issuing the rating" |
| 120 | `Rating` xs:string required | §6.6.2 | matches | "shall contain a human-readable representation of the rating" |
| 121-129 | `Audience` simpleContent with `scope` required attribute | §6.6.2.1 | matches | structural |
| 130 | `<xs:any namespace="##other" processContents="lax">` | §6.6 prose | matches | "allows information, beyond the textual representation of the rating contained in the Rating element, to be associated" |
| 133-138 | `EssenceDescriptorBaseType` | §6.7 (Table 11) | matches | structural |
| 135 | `Id` UUIDType required | §6.7.1 | matches | "shall uniquely identify the EssenceDescriptor element" |
| 136 | `<xs:any namespace="##other">` essence descriptor body | §6.1.10.1 | matches | placeholder for Track File-specific descriptor; "specified by the defining specification of each Track File" |
| 139-145 | `ContentVersionType` | §6.8 (Table 12) | matches | structural |
| 141 | `Id` xs:anyURI required | §6.8.1 | matches | "shall identify the content represented by the Composition Playlist" |
| 142 | `LabelText` UserTextType required | §6.8.2 | matches | "human-readable description of the content" |
| 146-159 | `SegmentType` | §6.9 (Table 13) | matches | structural |
| 148 | `Id` UUIDType required | §6.9.1 | matches | "shall uniquely identify the Segment for asset management purposes" |
| 149 | `Annotation` UserTextType minOccurs=0 | §6.9.2 | matches | "free-form, human-readable annotation describing the Segment" |
| 150-157 | `SequenceList` required, with MarkerSequence + xs:any | §6.9.3 | matches | structural |
| 153 | `MarkerSequence` SequenceType minOccurs=0 | §6.9.3.1 | matches | "defines markers" |
| 154 | `<xs:any namespace="##other">` Extension Sequences | §6.9.3 | matches | "Extension Sequences ... derived, directly or indirectly, from SequenceType" |
| 160-172 | `SequenceType` | §6.10 (Table 14) | matches | structural |
| 162 | `Id` UUIDType required | §6.10.1 | matches | "shall uniquely identify the Sequence" |
| 163 | `TrackId` UUIDType required | §6.10.2 | matches | "shall uniquely identify the Virtual Track to which the Sequence belongs" |
| 164-170 | `ResourceList` required, Resource maxOccurs=unbounded | §6.10.3 | matches | "elements of ResourceList shall be ordered" |
| 173-183 | `BaseResourceType abstract="1"` | §6.11 (Table 15) | matches | abstract base; concrete subtypes referenced via xsi:type |
| 175 | `Id` UUIDType required | §6.11.1 | matches | "shall uniquely identify this specific Resource instance" |
| 176 | `Annotation` UserTextType minOccurs=0 | §6.11.2 | matches | "describing the Resource" |
| 177 | `EditRate` RationalType minOccurs=0 | §6.11.3 | matches | absence rule in §6.11.3: equals Composition Edit Rate |
| 178 | `IntrinsicDuration` xs:nonNegativeInteger required | §6.11.4 | matches | "shall be the native duration of the underlying Asset in Resource Edit Units" |
| 179 | `EntryPoint` xs:nonNegativeInteger minOccurs=0 | §6.11.5 | matches | absence rule (= 0) in §6.11.5 |
| 180 | `SourceDuration` xs:nonNegativeInteger minOccurs=0 | §6.11.6 | under-constrained | XSD: any non-neg int. Prose: "shall be between 0 (zero) and IntrinsicDuration – EntryPoint" |
| 181 | `RepeatCount` xs:positiveInteger minOccurs=0 | §6.11.7 | matches | absence rule (= 1) in §6.11.7 |
| 184-195 | `TrackFileResourceType` extends BaseResourceType | §6.12 (Table 16) | matches | structural |
| 188 | `SourceEncoding` UUIDType required | §6.12.1 | matches | "shall reference one element of the EssenceDescriptorList through its Id element" |
| 189 | `TrackFileId` UUIDType required | §6.12.2 | matches | "shall uniquely identify the underlying Track File" |
| 190 | `KeyId` UUIDType minOccurs=0 | §6.12.3 | under-constrained | XSD: optional unconditionally. Prose: "shall be present if any portion of the underlying track file is encrypted" — conditional cardinality |
| 191 | `Hash` xs:base64Binary minOccurs=0 | §6.12.4 | under-constrained | XSD: any base64. Prose: "shall be computed by applying the SHA-256 message digest algorithm" |
| 196-204 | `MarkerResourceType` extends BaseResourceType | §6.13 (Table 17) | matches | structural |
| 200 | `Marker` MarkerType minOccurs=0 maxOccurs=unbounded | §6.13 | matches | "each content marker shall be represented by a Marker element of type MarkerType" |
| 205-219 | `MarkerType` | §6.14 (Table 18) | matches | structural |
| 207 | `Annotation` UserTextType minOccurs=0 | §6.14.1 | matches | "free-form, human-readable annotation describing the composition" |
| 208-217 | `Label` simpleContent with `scope` attribute default | §6.14.2 | matches | scope default URI is the normative trigger for Table 19 value-set per §6.14.2 |
| 218 | `Offset` xs:nonNegativeInteger required | §6.13 | matches | "position of each Marker is determined by its offset from the start of the timeline" |

## B. Prose constraints not in XSD

| Prose § | Constraint | Category | Notes |
|---|---|---|---|
| §5.1 | "In the event of a conflict between schema definitions and the prose, the prose shall take precedence" | meta-rule | XSD cannot self-declare its own precedence |
| §5.2 | CPL "shall be encoded using the UTF-8 character encoding" | encoding | XML parsers honor the encoding declaration; XSD does not enforce |
| §5.3 | MIME type "shall be text/xml" | transport metadata | not in XML body |
| §5.4 | Modified-schema instances "shall use a different namespace and no two distinct schemas shall have the same target namespace" | versioning meta-rule | rule about schema authoring, not a per-instance check |
| §6.1.1 | "Any two Composition Playlist instances may have identical Id values if and only if the two Composition Playlist instances are identical" | cross-document identity | invariant across multiple CPLs |
| §6.1.9 | "No two ContentVersion elements shall have identical Id elements" | within-doc uniqueness | xs:unique could express this; XSD does not |
| §6.1.9 | "Two Composition Playlist instances shall be assumed to refer to the same content if they have in common at least one Id element of a ContentVersion element" | cross-document semantic | content-identity invariant |
| §6.1.9 / §6.1.15 / §6.9.3 | "Implementations shall ignore any [extension elements] from a namespace it does not recognize" | implementer runtime behavior | not a validation rule |
| §6.1.10 | "In case of a conflict between an EssenceDescriptor element and descriptive information in the underlying Track File, the latter shall take precedence" | conflict precedence | cross-document resolution rule |
| §6.1.10.1 | "Each EssenceDescriptor shall be referenced through its Id element by at least one Resource of type derived from TrackFileResource" | reachability | structural graph constraint within CPL |
| §6.1.13 | TotalRunningTime "shall indicate the approximate duration"; "Exact running time ... shall be calculated as specified in Section 7.1, and shall always take precedence in case of conflict" | informational + computed | declared informative; conflict-resolution against §7.1 calculation |
| §6.1.16 | SegmentList "shall contain an ordered list of Segment elements" | playback ordering semantics | XSD enforces XML order via xs:sequence; prose adds playback semantics |
| §6.1.17 | "If the Signer element is present, then the Signature element shall also be present" | conditional cardinality | XSD has both minOccurs=0; conditional dependency not expressible |
| §6.1.17 | Signer when X.509 used: "shall contain one X509Data element containing one X509IssuerSerial element" | conditional structural | conditional sub-structure for an extension-namespaced element |
| §6.1.18 | "If the Signature element is present, then the Signer element shall be present" | conditional cardinality | mirror of §6.1.17 |
| §6.1.18 | Signature "shall be enveloped, as specified in [XML Digital Signature], and apply to the entire Composition Playlist" | crypto invariant | external standard ref |
| §6.2.1.2 | TimecodeRate "shall specify the nearest integer frames per second rate of the timecode" | semantic interpretation | XSD: positive int; meaning per §6.2.1.2 |
| §6.3 | TimecodeStartAddress fields "shall be the hours / minutes / seconds / frames field" | semantic interpretation | XSD pattern parses the string; meaning is in prose |
| §6.5.2 | Language values "shall be a Language Tag as specified in [RFC 5646]" | external value-set | BCP-47 |
| §6.5.3 | Region values "shall be a valid region subtag of the Language Subtag Registry" | external value-set | RFC 5646 subtag registry |
| §6.5.4 | "There shall be only one ContentMaturityRating element with a given value of the Agency element" | within-Locale uniqueness | xs:unique could express; XSD does not |
| §6.6.2.1 | Audience `scope` attribute "shall determine the permissible values of the Audience element" | scope-keyed value-set | per-scope value sets defined out-of-band |
| §6.9.3 | "All Sequences with equal TrackId shall belong to the same Virtual Track" | cross-segment grouping | XSD has no view of cross-segment structure |
| §6.9.3 | "A given TrackId value shall be used by only one Sequence in each Segment and, if used in one Segment, it shall be used by exactly one Sequence in all other Segments" | cross-segment uniqueness + presence | strong invariant across segments |
| §6.9.3 | "Each Sequence and Virtual Track shall be associated with a single aspect of the presentation and therefore a single kind of essence" | semantic invariant | requires essence-type knowledge |
| §6.10 | "All Resources elements within a Sequence shall be of the same type, as defined in [XML Schema Part 1: Structures]" | cross-element type consistency | XSD allows mixed Resource subtypes within a Sequence; prose forbids |
| §6.11.4 | "The defining specification BaseResourceType subclasses shall specify the native duration of the Asset" | external spec delegation | cross-document semantic |
| §6.11.5 | EntryPoint absence "a value of 0 shall be assumed" | semantic default | XSD has no default attribute; prose specifies absence semantics |
| §6.11.6 | SourceDuration "shall be between 0 (zero) and IntrinsicDuration – EntryPoint" | computed bound | cross-field math relationship |
| §6.11.6 | SourceDuration absence "shall be equal to IntrinsicDuration – EntryPoint" | semantic default | XSD has no default; computed from other fields |
| §6.11.7 | RepeatCount absence "shall be equal to one" | semantic default | XSD has no default for elements |
| §6.12.2 | "The defining specification for each Track File referenced in a Composition Playlist shall specify the identifier for use with TrackFileId" | external spec delegation | cross-document semantic |
| §6.12.3 | KeyId "shall be present if any portion of the underlying track file is encrypted" | conditional cardinality | depends on external Track File property |
| §6.12.4 | Hash "shall be computed by applying the SHA-256 message digest algorithm [RFC 4634] over the entire Track File" | computation algorithm | cross-document; algorithm not in XSD |
| §6.13 | MarkerResourceType "native start point ... shall be the start of the timeline" | semantic invariant | not in XSD |
| §6.13 | MarkerResource IntrinsicDuration "shall be set to any value equal or larger to the largest Offset value within all its Marker elements" | cross-field bound | IntrinsicDuration ≥ max(Offset) over child Markers |
| §6.14.2 | Label default-scope value "shall match one of the values listed in Table 19" | conditional value-set | scope-keyed enumeration; XSD has scope default but not the value-set |

## C. Conflicts (prose vs XSD)

_None found._ §5.1 provides the precedence rule; under-constrained
status rows in Section A are not conflicts but cases where prose
tightens what XSD encodes.

## D. Gaps

| Item | Reason |
|---|---|
| XSD line 8 `<xs:element name="CompositionPlaylist">` global declaration | §5 says CPL "consists of a single CompositionPlaylist element" — global vs in-scope distinction not addressed |
| XSD lines 53 / 130 / 136 / 143 / 154 `processContents="lax"` on `<xs:any>` | XSD validation mode for unknown extensions; §6.1.15 only says "implementations shall ignore" — schema vocabulary choice, not specified in prose |
| XSD line 173 `BaseResourceType abstract="1"` + xsi:type machinery | §6.11 says "abstract base class for Resources" but does not specify the xsi:type pattern operators must use — XSD mechanism not described in prose |

These are XSD-mechanism choices the spec author made when encoding
the schema. They become operationally normative because the schema
is normative (§5.1), but the prose does not discuss them. Not
defects; documented here as "schema-only by mechanism, not by §6
delegation."

## Catalogue coverage cross-check

Per `crates/imferno-core/src/cpl/codes.rs`, the engine emits these
ST 2067-3:2013 codes. Mapped against Section B rows:

| Section B row | Implemented? | Catalogue code (if yes) |
|---|---|---|
| §6.1.9 ContentVersion Id uniqueness | yes | `St2067_3_2013::ContentVersionIdDuplicate` |
| §6.1.10.1 EssenceDescriptor reachability | yes | `St2067_3_2013::SourceEncodingUnresolved` (and `SourceEncodingNoEssenceDescriptorList`, `EssenceDescriptorListEmpty`) |
| §6.1.15 ContentVersionList empty (structural) | yes | `St2067_3_2013::ContentVersionListEmpty` |
| §6.5.2 RFC 5646 Language Tag | yes | `St2067_3_2013::LocaleLanguageTagInvalid` |
| §6.5.3 Region subtag | partial | covered by `LocaleLanguageTagInvalid`'s parser; no separate code |
| §6.9.3 cross-segment TrackId uniqueness | yes | `St2067_3_2013::TrackIdNotUnique` |
| §6.1.8 / Table 3 ContentKind value-set | yes | `St2067_3_2013::ContentKindUnknown` |
| §6.14.2 / Table 19 Marker Label value-set | yes | `St2067_3_2013::MarkerLabelUnknown` |
| §6.13 Marker offset ≤ resource duration | yes | `St2067_3_2013::MarkerOffsetOutOfRange` |
| §7.2.2 / §7.3 segment duration consistency | yes | `St2067_3_2013::SegmentDuration`, `SegmentDurationIntegerEditUnits` |
| §6.8.1 ContentVersion Id non-empty | yes | `St2067_3_2013::ContentVersionIdInvalid` |
| §6.8.2 ContentVersion LabelText required | yes | `St2067_3_2013::ContentVersionLabelTextMissing` |
| §5.1 prose precedence (meta) | n/a | not a validation rule |
| §5.2 UTF-8 encoding | gap | not enforced |
| §5.3 text/xml MIME | gap | not enforced (PKL-side concern) |
| §5.4 namespace-change rule | n/a | meta-rule for spec authors |
| §6.1.1 cross-CPL Id identity invariant | gap | not enforced |
| §6.1.9 cross-CPL ContentVersion synonymy | n/a | "shall be assumed" — informational |
| §6.1.10 EssenceDescriptor vs Track File precedence | n/a | conflict-resolution rule |
| §6.1.13 TotalRunningTime vs §7.1 calculation | gap | not cross-checked |
| §6.1.16 SegmentList playback ordering | n/a | XML order = playback order, enforced implicitly |
| §6.1.17 / §6.1.18 Signer ↔ Signature pairing | gap | conditional cardinality not enforced |
| §6.1.17 X.509 sub-structure | gap | XML Digital Signature path-level check not performed |
| §6.1.18 enveloped signature scope | gap | crypto verification not performed |
| §6.2.1.2 TimecodeRate "nearest integer fps" | gap | semantic; not checked against EditRate-derived integer rounding |
| §6.3 TimecodeType field semantics | n/a | XSD pattern handles syntactic; semantics are interpretive |
| §6.5.4 ContentMaturityRating Agency uniqueness | gap | within-Locale uniqueness not enforced |
| §6.6.2.1 Audience scope-keyed value-set | n/a | out of scope per §6.6.2.1 |
| §6.9.3 Virtual-Track essence-kind invariant | gap | requires essence-type plumbing |
| §6.10 Same Resource type within Sequence | gap | not enforced |
| §6.11.4 / §6.12.2 external spec delegation | n/a | cross-document; out of engine scope |
| §6.11.5 EntryPoint default 0 | n/a | absence semantic, applied transparently |
| §6.11.6 SourceDuration range bound | gap | computed bound 0 ≤ SourceDuration ≤ IntrinsicDuration – EntryPoint not enforced |
| §6.11.6 SourceDuration absence default | n/a | absence semantic, applied transparently |
| §6.11.7 RepeatCount absence default 1 | n/a | absence semantic, applied transparently |
| §6.12.3 KeyId conditional presence | gap | requires Track File encryption check (cross-document) |
| §6.12.4 Hash SHA-256 algorithm | gap | hash recomputation against Track File not performed |
| §6.13 MarkerResource native start point = 0 | n/a | informational/structural |
| §6.13 IntrinsicDuration ≥ max(Offset) | gap | cross-field bound not enforced |

**Coverage summary**: ~12 prose constraints implemented, ~12 explicit
gaps that could be added to the catalogue, ~13 not-applicable
(meta-rules, conflict-resolution, out-of-scope delegations).
