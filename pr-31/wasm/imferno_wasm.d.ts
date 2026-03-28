/* tslint:disable */
/* eslint-disable */
export interface Asset {
    id: ImfUuid;
    packing_list: boolean | null;
    chunk_list: ChunkList;
}

export interface AssetList {
    assets: Asset[];
}

export interface AssetMap {
    id: ImfUuid;
    annotation_text: string | null;
    creator: string | null;
    volume_count: number;
    issue_date: string;
    issuer: string | null;
    asset_list: AssetList;
}

export interface AudienceElement {
    @scope?: string | null;
    $text?: string | null;
}

export interface AudioSubDescriptors {
    SoundfieldGroupLabelSubDescriptor?: SoundfieldGroupLabelSubDescriptor | null;
}

export interface CDCIDescriptor {
    InstanceUID?: string | null;
    StoredWidth?: number | null;
    StoredHeight?: number | null;
    DisplayWidth?: number | null;
    DisplayHeight?: number | null;
    ActiveWidth?: number | null;
    ActiveHeight?: number | null;
    SampleRate?: EditRate | null;
    ImageAspectRatio?: string | null;
    ColorPrimaries?: ColorPrimaries | null;
    TransferCharacteristic?: TransferCharacteristic | null;
    CodingEquations?: CodingEquations | null;
    PictureCompression?: VideoCodec | null;
    ComponentDepth?: number | null;
    FrameLayout?: string | null;
    DisplayF2Offset?: number | null;
    HorizontalSubsampling?: number | null;
    VerticalSubsampling?: number | null;
    ColorSiting?: number | null;
    BlackRefLevel?: number | null;
    WhiteRefLevel?: number | null;
    ColorRange?: number | null;
    StoredF2Offset?: number | null;
    SampledWidth?: number | null;
    SampledHeight?: number | null;
    SampledXOffset?: number | null;
    SampledYOffset?: number | null;
    AlphaTransparency?: string | null;
    ImageAlignmentOffset?: number | null;
    ImageStartOffset?: number | null;
    ImageEndOffset?: number | null;
    FieldDominance?: number | null;
    ReversedByteOrder?: string | null;
    PaddingBits?: number | null;
    AlphaSampleDepth?: number | null;
    LinkedTrackID?: number | null;
    SubDescriptors?: VideoSubDescriptors | null;
}

export interface Chunk {
    path: string;
    volume_index: number;
}

export interface ChunkList {
    chunks: Chunk[];
}

export interface CompositionPlaylist {
    id: ImfUuid;
    annotation?: LanguageString | null;
    issueDate: string;
    issuer?: LanguageString | null;
    creator?: LanguageString | null;
    contentOriginator?: LanguageString | null;
    contentTitle: LanguageString;
    contentKind?: ContentKindElement;
    contentVersionList?: ContentVersionList | null;
    essenceDescriptorList?: EssenceDescriptorList | null;
    editRate?: EditRate | null;
    totalRunningTime?: string | null;
    localeList?: LocaleList | null;
    extensionProperties?: ExtensionProperties | null;
    compositionTimecode?: CompositionTimecode | null;
    segmentList: SegmentList;
}

export interface CompositionTimecode {
    timecodeDropFrame: boolean | null;
    timecodeRate: number | null;
    timecodeStartAddress: string | null;
}

export interface ContainerConstraintsSubDescriptor {
    InstanceID?: string | null;
}

export interface ContentKindElement {
    kind: ContentKind;
    scope: string | null;
}

export interface ContentMaturityRating {
    agency: string;
    rating?: string | null;
    audience?: AudienceElement | null;
}

export interface ContentMaturityRatingList {
    contentMaturityRatings: ContentMaturityRating[];
}

export interface ContentVersion {
    id: string;
    labelText?: LanguageString | null;
}

export interface ContentVersionList {
    contentVersions: ContentVersion[];
}

export interface DCTimedTextDescriptor {
    InstanceID?: string | null;
    LinkedTrackID?: number | null;
    SampleRate?: EditRate | null;
    RFC5646LanguageTagList?: LanguageTag[];
    NamespaceURI?: string | null;
}

export interface EditRate {
    numerator: number;
    denominator: number;
}

export interface EssenceDescriptor {
    id: ImfUuid;
    rgbaDescriptor?: RGBADescriptor | null;
    cdciDescriptor?: CDCIDescriptor | null;
    wavePCMDescriptor?: WAVEPCMDescriptor | null;
    dcTimedTextDescriptor?: DCTimedTextDescriptor | null;
    iabEssenceDescriptor?: IABEssenceDescriptor | null;
    isxdDataEssenceDescriptor?: ISXDDataEssenceDescriptor | null;
}

export interface EssenceDescriptorList {
    essenceDescriptors: EssenceDescriptor[];
}

export interface ExtensionProperties {
    applicationIdentification?: string | null;
    maxCLL?: number | null;
    maxFALL?: number | null;
}

export interface ForcedNarrativeSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface HearingImpairedCaptionsSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface IABEssenceDescriptor {
    InstanceID?: string | null;
    LinkedTrackID?: number | null;
    SampleRate?: EditRate | null;
    AudioSampleRate?: EditRate | null;
    ChannelCount?: number | null;
    QuantizationBits?: number | null;
    ContainerFormat?: string | null;
    SoundCompression?: string | null;
    Codec?: string | null;
    ElectrospatialFormulation?: number | null;
    SubDescriptors?: IABSubDescriptors | null;
}

export interface IABSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface IABSoundfieldLabelSubDescriptor {
    InstanceID?: string | null;
    MCATagSymbol?: McaTagSymbol | null;
    MCATagName?: string | null;
    MCALabelDictionaryID?: string | null;
    RFC5646SpokenLanguage?: LanguageTag | null;
}

export interface IABSubDescriptors {
    IABSoundfieldLabelSubDescriptor?: IABSoundfieldLabelSubDescriptor | null;
}

export interface ISXDDataEssenceDescriptor {
    InstanceID?: string | null;
    LinkedTrackID?: number | null;
    SampleRate?: EditRate | null;
    DataEssenceCoding?: string | null;
    NamespaceURI?: string | null;
    SubDescriptors?: IsxdSubDescriptors | null;
}

export interface ISXDSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface IsxdSubDescriptors {
    ContainerConstraintsSubDescriptor?: ContainerConstraintsSubDescriptor | null;
}

export interface J2CLayout {
    RGBAComponent?: RGBALayoutComponent[];
}

export interface J2KComponentSizing {
    Ssiz?: number | null;
    XRSiz?: number | null;
    YRSiz?: number | null;
}

export interface J2KExtendedCapabilities {
    Pcap?: number | null;
}

export interface JPEG2000SubDescriptor {
    InstanceID?: string | null;
    Rsiz?: number | null;
    Xsiz?: number | null;
    Ysiz?: number | null;
    XOsiz?: number | null;
    YOsiz?: number | null;
    XTsiz?: number | null;
    YTsiz?: number | null;
    XTOsiz?: number | null;
    YTOsiz?: number | null;
    Csiz?: number | null;
    CodingStyleDefault?: string | null;
    QuantizationDefault?: string | null;
    J2CLayout?: J2CLayout | null;
    J2KExtendedCapabilities?: J2KExtendedCapabilities | null;
    PictureComponentSizing?: PictureComponentSizing | null;
}

export interface LanguageList {
    languages: LanguageTag[];
}

export interface LanguageString {
    text: string;
    language: LanguageTag | null;
}

export interface Locale {
    languageList?: LanguageList | null;
    regionList?: RegionList | null;
    contentMaturityRatingList?: ContentMaturityRatingList | null;
}

export interface LocaleList {
    locales: Locale[];
}

export interface MainAudioSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface MainImageSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface MarkerInfo {
    annotation?: string | null;
    label: MarkerLabelElement;
    offset: number;
}

export interface MarkerLabelElement {
    label: MarkerLabel;
    scope: string | null;
}

export interface MarkerSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface PHDRMetadataTrackSubDescriptor {
    InstanceID?: string | null;
    PHDRMetadataTrackSubDescriptor_DataDefinition?: string | null;
    PHDRMetadataTrackSubDescriptor_SimplePayloadSID?: number | null;
    PHDRMetadataTrackSubDescriptor_SourceTrackID?: number | null;
}

export interface PictureComponentSizing {
    J2KComponentSizing?: J2KComponentSizing[];
}

export interface RGBADescriptor {
    InstanceID?: string | null;
    DisplayWidth?: number | null;
    DisplayHeight?: number | null;
    StoredWidth?: number | null;
    StoredHeight?: number | null;
    SampleRate?: EditRate | null;
    ImageAspectRatio?: string | null;
    ColorPrimaries?: ColorPrimaries | null;
    TransferCharacteristic?: TransferCharacteristic | null;
    CodingEquations?: CodingEquations | null;
    PictureCompression?: VideoCodec | null;
    FrameLayout?: string | null;
    DisplayF2Offset?: number | null;
    ComponentMaxRef?: number | null;
    ComponentMinRef?: number | null;
    ScanningDirection?: string | null;
    StoredF2Offset?: number | null;
    SampledWidth?: number | null;
    SampledHeight?: number | null;
    SampledXOffset?: number | null;
    SampledYOffset?: number | null;
    AlphaTransparency?: string | null;
    ImageAlignmentOffset?: number | null;
    ImageStartOffset?: number | null;
    ImageEndOffset?: number | null;
    FieldDominance?: number | null;
    AlphaMaxRef?: number | null;
    AlphaMinRef?: number | null;
    Palette?: string | null;
    PaletteLayout?: string | null;
    LinkedTrackID?: number | null;
    SubDescriptors?: VideoSubDescriptors | null;
}

export interface RGBALayoutComponent {
    Code?: string;
    ComponentSize?: number;
}

export interface RegionList {
    regions: string[];
}

export interface Resolution {
    width: number;
    height: number;
}

export interface Resource {
    id: ImfUuid;
    annotation?: LanguageString | null;
    editRate?: EditRate | null;
    intrinsicDuration: number;
    entryPoint?: number | null;
    sourceDuration?: number | null;
    sourceEncoding?: ImfUuid | null;
    trackFileId?: ImfUuid | null;
    repeatCount?: number | null;
    keyId?: ImfUuid | null;
    hash?: string | null;
    markers?: MarkerInfo[];
}

export interface ResourceList {
    resources?: Resource[];
}

export interface Segment {
    id: ImfUuid;
    sequenceList: SequenceList;
}

export interface SegmentList {
    segments: Segment[];
}

export interface SequenceList {
    markerSequences?: MarkerSequence[];
    mainImageSequences?: MainImageSequence[];
    mainAudioSequences?: MainAudioSequence[];
    subtitlesSequence?: SubtitlesSequence[];
    hearingImpairedCaptionsSequence?: HearingImpairedCaptionsSequence[];
    forcedNarrativeSequence?: ForcedNarrativeSequence[];
    iabSequences?: IABSequence[];
    isxdSequences?: ISXDSequence[];
}

export interface SoundfieldGroupLabelSubDescriptor {
    MCATagSymbol?: McaTagSymbol | null;
    MCATagName?: string | null;
    MCAAudioContentKind?: string | null;
    RFC5646SpokenLanguage?: LanguageTag | null;
}

export interface SubtitlesSequence {
    id: ImfUuid;
    trackId: ImfUuid;
    resourceList: ResourceList;
}

export interface TrackInfo {
    track_id: string;
    track_type: string;
    codec: string;
    language: string | null;
    channels: string | null;
    format_details: string | null;
    resolution: string | null;
    framerate: string | null;
    bit_depth: string | null;
    subtitle_type: string | null;
}

export interface VideoSubDescriptors {
    PHDRMetadataTrackSubDescriptor?: PHDRMetadataTrackSubDescriptor | null;
    JPEG2000SubDescriptor?: JPEG2000SubDescriptor | null;
}

export interface VolumeIndex {
    Index: number;
}

export interface WAVEPCMDescriptor {
    InstanceID?: string | null;
    SampleRate?: EditRate | null;
    AudioSampleRate?: EditRate | null;
    ChannelCount?: number | null;
    QuantizationBits?: number | null;
    LinkedTrackID?: number | null;
    SubDescriptors?: AudioSubDescriptors | null;
}

export type CodingEquations = "Bt601" | "Bt709" | "Bt2020Ncl" | { Unknown: string };

export type ColorPrimaries = "Bt601_625" | "Bt601_525" | "Bt709" | "Bt2020" | "DciP3" | "P3D65" | { Unknown: string };

export type ContentKind = "Feature" | "Trailer" | "Test" | "Promo" | "Teaser" | "RatingBump" | "Advertisement" | "Episode" | "Short" | "Commercial" | "PublicServiceAnnouncement" | { Other: string };

export type CplNamespace = "Smpte2067_3_2013" | "Smpte2067_3_2016" | "Smpte2067_3_2020" | "Dci429_7" | { Unknown: string };

export type LanguageTag = string;

export type MarkerLabel = "Ffoc" | "Lfoc" | "Ffac" | "Lfac" | "Ffmc" | "Lfmc" | "Ffhc" | "Lfhc" | { Other: string };

export type McaTagSymbol = "Sg51" | "Sg71" | "Sg71Ds" | "SgSt" | "SgMono" | "Iab" | "Left" | "Right" | "Center" | "Lfe" | "LeftSurround" | "RightSurround" | "LeftSideSurround" | "RightSideSurround" | "LeftRearSurround" | "RightRearSurround" | { Other: string };

export type TransferCharacteristic = "Linear" | "Bt709" | "Smpte240M" | "XvYcc709" | "Bt2020" | "PqSt2084" | "Hlg" | { Unknown: string };

export type VideoCodec = "Jpeg2000" | "Jpeg2000Imf2k" | "Jpeg2000Imf4k" | "Jpeg2000Broadcast" | "Jpeg2000Ht" | "Vc5" | "Mpeg2" | "H264" | "H265" | "ProRes" | "Av1" | { Unknown: string };


/**
 * Build a structured report from an IMF package.
 *
 * Pass all XML files as a plain JS object where each key is the filename
 * and each value is the file's text content.
 *
 * Returns an `ImfReport` containing package metadata, CPL analysis, and
 * validation results. This is the same JSON that the CLI `export` command produces.
 *
 * Options (all optional):
 * - `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
 * - `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`
 * - `rules`: ESLint-style rules configuration object
 */
export function buildReport(files: any, options: any): any;

/**
 * Format a previously built `ImfReport` as a human-readable string.
 *
 * Pass the object returned by `buildReport()` (or any valid `ImfReport` JSON).
 * Returns the same output as `imferno report` on the CLI.
 */
export function formatReport(report: any): string;

/**
 * Get library version
 */
export function getVersion(): string;

/**
 * Initialize the WASM module
 */
export function init(): void;

/**
 * Parse an IMF package from in-memory files, returning the full parsed package.
 *
 * Unlike `buildReport` which returns a summary, this returns the complete
 * `Imferno` struct with all essence descriptors, locales, content versions, etc.
 */
export function parsePackage(files: any): any;

/**
 * Parse and validate an IMF package in one call.
 *
 * Returns `{ package, validation }` where `package` is the full `Imferno`
 * struct and `validation` is the `ValidationReport` with all findings.
 *
 * This is the recommended entry point.
 */
export function validate(files: any, options: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly buildReport: (a: any, b: any) => [number, number, number];
    readonly formatReport: (a: any) => [number, number, number, number];
    readonly getVersion: () => [number, number];
    readonly init: () => void;
    readonly parsePackage: (a: any) => [number, number, number];
    readonly validate: (a: any, b: any) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
