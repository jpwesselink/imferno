import { useState, useRef, useCallback, useEffect } from 'react';

// ─── configurable rules ───────────────────────────────────────────────────────

type RuleSeverity = 'off' | 'info' | 'warn' | 'error' | 'critical';

const CONFIGURABLE_RULES: Array<{ code: string; label: string; hint: string }> = [
    { code: 'ST2067-2:2020:8.3/FileNotFound',        label: 'FileNotFound',      hint: 'Asset file not found on disk' },
    { code: 'ST2067-3:2020:7.2.2/SegmentDuration',   label: 'SegmentDuration',   hint: 'Unequal segment durations across tracks' },
];
import IMFPackageViewer from './IMFPackageViewer';

// ─── global WASM handle ───────────────────────────────────────────────────────

declare global {
    interface Window {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        __imfWasm?: any;
        __imfWasmError?: string;
    }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function getWasmModule(): any | null {
    return typeof window !== 'undefined' ? window.__imfWasm ?? null : null;
}

// ─── types ────────────────────────────────────────────────────────────────────

type FileKind = 'volindex' | 'assetmap' | 'cpl' | 'pkl' | 'opl';

interface UploadedFile {
    uid: string;
    name: string;
    size: number;
    kind: FileKind | 'unknown';
}

// ─── helpers ──────────────────────────────────────────────────────────────────

const MAX_BYTES = 2 * 1024 * 1024; // 2 MB

function detectKind(name: string): FileKind | 'unknown' {
    const u = name.toUpperCase();
    if (u === 'VOLINDEX.XML') return 'volindex';
    if (u === 'ASSETMAP.XML') return 'assetmap';
    if (u.startsWith('PKL_') && u.endsWith('.XML')) return 'pkl';
    if (u.startsWith('OPL_') && u.endsWith('.XML')) return 'opl';
    if (u.endsWith('.XML')) return 'cpl';
    return 'unknown';
}

function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1_048_576) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / 1_048_576).toFixed(1)} MB`;
}

function makeUid() {
    return Math.random().toString(36).slice(2);
}

// ─── map parsePackage + buildReport → IMFPackageViewer data ───────────────────

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mapToViewData(pkg: any, report: any): any {
    const v = report.validation;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const flatIssues: any[] = [
        ...(v.critical ?? []),
        ...(v.errors ?? []),
        ...(v.warnings ?? []),
        ...(v.info ?? []),
    ];

    const valid = !(v.critical?.length > 0) && !(v.errors?.length > 0);

    // Build essence descriptor lookup per CPL (id → descriptor)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function buildDescriptorMap(cpl: any): Record<string, any> {
        const map: Record<string, any> = {}; // eslint-disable-line @typescript-eslint/no-explicit-any
        for (const ed of cpl.EssenceDescriptorList?.EssenceDescriptor ?? cpl.essenceDescriptorList?.essenceDescriptors ?? []) {
            const id = ed.Id ?? ed.id;
            if (id) map[id] = ed;
        }
        return map;
    }

    // Extract language from an essence descriptor
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function langFromDescriptor(ed: any): string | null {
        // Audio: WAVEPCMDescriptor → SubDescriptors → SoundfieldGroupLabelSubDescriptor
        const wave = ed.WAVEPCMDescriptor ?? ed.wavePcmDescriptor;
        const sf = wave?.SubDescriptors?.SoundfieldGroupLabelSubDescriptor ?? wave?.subDescriptors?.soundfieldGroupLabelSubDescriptor;
        const lang = sf?.RFC5646SpokenLanguage ?? sf?.rfc5646SpokenLanguage;
        if (lang) return typeof lang === 'string' ? lang : lang.toString();
        // Timed text
        const tt = ed.DCTimedTextDescriptor ?? ed.dcTimedTextDescriptor;
        const ttLangs = tt?.RFC5646LanguageTagList ?? tt?.rfc5646LanguageTagList;
        if (Array.isArray(ttLangs) && ttLangs.length > 0) return ttLangs[0].toString();
        // IAB
        const iab = ed.IABEssenceDescriptor ?? ed.iabEssenceDescriptor;
        const iabSf = iab?.SubDescriptors?.IABSoundfieldLabelSubDescriptor ?? iab?.subDescriptors?.iabSoundfieldLabelSubDescriptor;
        const iabLang = iabSf?.RFC5646SpokenLanguage ?? iabSf?.rfc5646SpokenLanguage;
        if (iabLang) return iabLang.toString();
        return null;
    }

    // Extract channel count from descriptor
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function channelsFromDescriptor(ed: any): number | null {
        const wave = ed.WAVEPCMDescriptor ?? ed.wavePcmDescriptor;
        if (wave?.ChannelCount ?? wave?.channelCount) return Number(wave.ChannelCount ?? wave.channelCount);
        const iab = ed.IABEssenceDescriptor ?? ed.iabEssenceDescriptor;
        if (iab?.ChannelCount ?? iab?.channelCount) return Number(iab.ChannelCount ?? iab.channelCount);
        return null;
    }

    // Extract soundfield from descriptor
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function soundfieldFromDescriptor(ed: any): string | null {
        const wave = ed.WAVEPCMDescriptor ?? ed.wavePcmDescriptor;
        const sf = wave?.SubDescriptors?.SoundfieldGroupLabelSubDescriptor ?? wave?.subDescriptors?.soundfieldGroupLabelSubDescriptor;
        const mca = sf?.MCATagSymbol ?? sf?.mcaTagSymbol;
        if (mca) return mca.toString();
        const name = sf?.MCATagName ?? sf?.mcaTagName;
        if (name) return name;
        const iab = ed.IABEssenceDescriptor ?? ed.iabEssenceDescriptor;
        if (iab) return 'Atmos';
        return null;
    }

    // Map the full Imferno CPLs
    const cplEntries = Object.entries(pkg.compositionPlaylists ?? {});

    return {
        package: {
            assetMapId: pkg.assetMap?.id ?? '',
            volumeIndex: pkg.volumeIndex?.index ?? 1,
            assetCount: pkg.assetMap?.assetList?.assets?.length ?? 0,
            cplCount: cplEntries.length,
            pklCount: Object.keys(pkg.packingLists ?? {}).length,
        },
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        cpls: cplEntries.map(([_uuid, cpl]: [string, any]) => {
            const er = cpl.EditRate ?? cpl.editRate;
            const editRate = er ? `${er.numerator ?? er.Numerator} ${er.denominator ?? er.Denominator}` : null;
            const descriptors = buildDescriptorMap(cpl);
            const contentTitle = cpl.ContentTitle?.text ?? cpl.contentTitle?.text ?? cpl.ContentTitle ?? cpl.contentTitle ?? '';
            const contentKind = cpl.ContentKind ?? cpl.contentKind ?? null;
            const segments = cpl.SegmentList?.Segment ?? cpl.segmentList?.segments ?? [];

            // Flatten sequences across all segments in CPL physical order
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const sequences: any[] = [];
            const seqTypeMap: Record<string, string> = {
                MainImageSequence: 'MainImageSequence',
                MainAudioSequence: 'MainAudioSequence',
                SubtitlesSequence: 'SubtitlesSequence',
                HearingImpairedCaptionsSequence: 'HearingImpairedCaptionsSequence',
                ForcedNarrativeSequence: 'ForcedNarrativeSequence',
                IABSequence: 'IABSequence',
                ISXDSequence: 'ISXDSequence',
                // camelCase variants (serde output)
                mainImageSequences: 'MainImageSequence',
                mainAudioSequences: 'MainAudioSequence',
                subtitlesSequences: 'SubtitlesSequence',
                hearingImpairedCaptionsSequences: 'HearingImpairedCaptionsSequence',
                forcedNarrativeSequences: 'ForcedNarrativeSequence',
                iabSequences: 'IABSequence',
                isxdSequences: 'ISXDSequence',
            };

            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            for (const seg of segments) {
                const sl = seg.SequenceList ?? seg.sequenceList ?? {};
                for (const [key, seqType] of Object.entries(seqTypeMap)) {
                    const seqArr = sl[key];
                    if (!Array.isArray(seqArr)) continue;
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    for (const seq of seqArr) {
                        const trackId = seq.TrackId ?? seq.trackId ?? '';
                        const seId = (seq.ResourceList?.Resource ?? seq.resourceList?.resources ?? [])[0]?.SourceEncoding ?? (seq.ResourceList?.Resource ?? seq.resourceList?.resources ?? [])[0]?.sourceEncoding;
                        const ed = seId ? descriptors[seId] : null;

                        // Check if we already have this track (merge resources across segments)
                        const existing = sequences.find((s: any) => s.trackId === trackId); // eslint-disable-line @typescript-eslint/no-explicit-any
                        // eslint-disable-next-line @typescript-eslint/no-explicit-any
                        const resources = (seq.ResourceList?.Resource ?? seq.resourceList?.resources ?? []).map((r: any) => ({
                            id: r.Id ?? r.id ?? '',
                            intrinsicDuration: r.IntrinsicDuration ?? r.intrinsicDuration ?? 0,
                            sourceDuration: r.SourceDuration ?? r.sourceDuration ?? r.IntrinsicDuration ?? r.intrinsicDuration ?? 0,
                            sourceEncoding: r.SourceEncoding ?? r.sourceEncoding ?? null,
                            trackFileId: r.TrackFileId ?? r.trackFileId ?? null,
                            editRate: (() => {
                                const re = r.EditRate ?? r.editRate;
                                if (re) return `${re.numerator ?? re.Numerator} ${re.denominator ?? re.Denominator}`;
                                return editRate;
                            })(),
                            entryPoint: r.EntryPoint ?? r.entryPoint ?? null,
                        }));

                        if (existing) {
                            existing.sequenceResources.push(...resources);
                        } else {
                            sequences.push({
                                type: seqType,
                                id: seq.Id ?? seq.id ?? '',
                                trackId,
                                language: ed ? langFromDescriptor(ed) : null,
                                channelCount: ed ? channelsFromDescriptor(ed) : null,
                                soundfield: ed ? soundfieldFromDescriptor(ed) : null,
                                segmentId: null,
                                sequenceNumber: sequences.length,
                                sequenceResources: resources,
                            });
                        }
                    }
                }
            }

            // Extract markers
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const markers: any[] = [];
            for (const seg of segments) {
                const sl = seg.SequenceList ?? seg.sequenceList ?? {};
                for (const ms of sl.MarkerSequence ?? sl.markerSequences ?? []) {
                    for (const r of ms.ResourceList?.Resource ?? ms.resourceList?.resources ?? []) {
                        for (const m of r.MarkerList ?? r.markerList ?? []) {
                            const label = m.Label?.value ?? m.label?.value ?? m.Label ?? m.label ?? '';
                            markers.push({
                                label: typeof label === 'string' ? label : String(label),
                                offset: m.Offset ?? m.offset ?? null,
                                scope: m.Label?.scope ?? m.label?.scope ?? null,
                            });
                        }
                    }
                }
            }

            return {
                id: cpl.Id ?? cpl.id ?? '',
                title: typeof contentTitle === 'string' ? contentTitle : String(contentTitle),
                applicationProfile: (() => {
                    for (const ext of cpl.ExtensionProperties?.extensionProperties ?? cpl.extensionProperties?.extensionProperties ?? []) {
                        const appId = ext.ApplicationIdentification ?? ext.applicationIdentification;
                        if (Array.isArray(appId) && appId.length > 0) return appId[0];
                        if (typeof appId === 'string') return appId;
                    }
                    return null;
                })(),
                segmentCount: segments.length,
                timecodeStart: (() => {
                    const tc = cpl.CompositionTimecode ?? cpl.compositionTimecode;
                    if (!tc) return null;
                    return tc.TimecodeStartAddress ?? tc.timecodeStartAddress ?? null;
                })(),
                isSupplemental: false,
                markers,
                sourceAsset: {
                    contentKind: typeof contentKind === 'string' ? contentKind : contentKind?.value ?? null,
                    contentTitle: typeof contentTitle === 'string' ? contentTitle : String(contentTitle),
                    territory: null,
                    editRate,
                    frameRate: editRate ? (() => {
                        const parts = editRate.split(/\s+/);
                        if (parts.length === 2) {
                            const fps = parseInt(parts[0]) / parseInt(parts[1]);
                            return fps ? Math.round(fps * 100) / 100 : null;
                        }
                        return null;
                    })() : null,
                    duration: null,
                    audioLanguages: [...new Set(sequences.filter(s => s.type === 'MainAudioSequence' && s.language).map(s => s.language))],
                    subtitleLanguages: [...new Set(sequences.filter(s => s.type === 'SubtitlesSequence' && s.language).map(s => s.language))],
                    forcedNarrativeLanguages: [...new Set(sequences.filter(s => s.type === 'ForcedNarrativeSequence' && s.language).map(s => s.language))],
                    audioType: null,
                    videoQuality: null,
                    videoDynamicRange: null,
                    tracks: { VIDEO: [], AUDIO: [], SUBTITLES: [], CAPTIONS: [], FORCED_NARRATIVE: [] },
                    sequences,
                },
            };
        }),
        validation: {
            valid,
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            issues: flatIssues.map((issue: any) => ({
                severity: String(issue.severity ?? ''),
                category: String(issue.category ?? ''),
                code: String(issue.code ?? ''),
                message: String(issue.message ?? ''),
                suggestion: issue.suggestion ?? null,
                cplId: issue.location?.cplId ?? issue.location?.cpl_id ?? undefined,
            })),
        },
    };
}

// ─── file card ────────────────────────────────────────────────────────────────

const KIND_LABEL: Record<FileKind, string> = {
    volindex: 'VOLINDEX',
    assetmap: 'ASSETMAP',
    cpl: 'CPL',
    pkl: 'PKL',
    opl: 'OPL',
};

function FileCard({ file, onRemove }: { file: UploadedFile; onRemove: () => void }) {
    return (
        <div style={{
            borderRadius: '12px',
            border: '1px solid var(--sl-color-hairline, #2e2e32)',
            background: 'var(--hp-card-bg, #202127)',
            overflow: 'hidden',
        }}>
            <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
                padding: '10px 16px',
            }}>
                {file.kind !== 'unknown' ? (
                    <span style={{
                        fontSize: '11px',
                        fontFamily: 'monospace',
                        fontWeight: 600,
                        padding: '2px 8px',
                        borderRadius: '4px',
                        background: 'rgba(194,97,38,0.2)',
                        color: 'var(--sl-color-accent, #f97316)',
                        flexShrink: 0,
                    }}>
                        {KIND_LABEL[file.kind]}
                    </span>
                ) : (
                    <span style={{ fontSize: '11px', color: 'var(--sl-color-gray-4, #6a6a71)', fontStyle: 'italic', flexShrink: 0 }}>unknown</span>
                )}
                <span style={{ fontSize: '13px', color: 'var(--sl-color-text, #dfdfd6)', fontWeight: 500, flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {file.name}
                </span>
                <span style={{ fontSize: '11px', color: 'var(--sl-color-gray-4, #6a6a71)', flexShrink: 0 }}>{formatBytes(file.size)}</span>
                <button
                    onClick={onRemove}
                    aria-label="Remove"
                    style={{
                        background: 'none',
                        border: 'none',
                        cursor: 'pointer',
                        color: 'var(--sl-color-gray-4, #6a6a71)',
                        fontSize: '16px',
                        lineHeight: 1,
                        padding: '0 2px',
                        flexShrink: 0,
                    }}
                >×</button>
            </div>
        </div>
    );
}

// ─── main component ───────────────────────────────────────────────────────────

export default function ImfPlayground() {
    const [files, setFiles] = useState<UploadedFile[]>([]);
    const [wasmReady, setWasmReady] = useState(false);
    const [wasmError, setWasmError] = useState<string | null>(null);
    const [dragging, setDragging] = useState(false);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const [packageData, setPackageData] = useState<any | null>(null);
    const [parseError, setParseError] = useState<string | null>(null);
    const [rulesConfig, setRulesConfig] = useState<Record<string, RuleSeverity>>({
        'ST2067-2:2020:8.3/FileNotFound': 'info',
    });
    const rulesConfigRef = useRef<Record<string, RuleSeverity>>({});
    rulesConfigRef.current = rulesConfig;
    const [showRules, setShowRules] = useState(false);
    const xmlMapRef = useRef<Record<string, string>>({});
    const inputRef = useRef<HTMLInputElement>(null);

    // Sync with the WASM module loaded by the is:inline script in index.astro.
    useEffect(() => {
        function check() {
            if (window.__imfWasm) {
                setWasmReady(true);
            } else if (window.__imfWasmError) {
                setWasmError(window.__imfWasmError);
            }
        }
        check();
        window.addEventListener('imf-wasm-ready', check, { once: true });
        return () => window.removeEventListener('imf-wasm-ready', check);
    }, []);

    // Re-validate whenever rules change (if files have already been parsed).
    useEffect(() => {
        const mod = getWasmModule();
        if (!mod || Object.keys(xmlMapRef.current).length === 0) return;
        try {
            const pkg = mod.parsePackage(xmlMapRef.current);
            const report = mod.buildReport(xmlMapRef.current, { rules: rulesConfig });
            setPackageData(mapToViewData(pkg, report));
            setParseError(null);
        } catch (e) {
            console.error('[imf] re-validate error:', e);
        }
    }, [rulesConfig]);

    const processFiles = useCallback(async (fileList: FileList | File[]) => {
        const list = Array.from(fileList).filter(
            (f) => f.name.toLowerCase().endsWith('.xml') && f.size <= MAX_BYTES,
        );
        if (list.length === 0) return;

        const entries: UploadedFile[] = list.map((f) => ({
            uid: makeUid(),
            name: f.name,
            size: f.size,
            kind: detectKind(f.name),
        }));
        setFiles(entries);
        setPackageData(null);
        setParseError(null);

        // Read all XML files
        const xmlMap: Record<string, string> = {};
        for (const f of list) {
            const kind = detectKind(f.name);
            if (kind !== 'unknown') {
                xmlMap[f.name] = await f.text();
            }
        }
        xmlMapRef.current = xmlMap;

        // Build report from all files at once
        const mod = getWasmModule();
        if (mod && Object.keys(xmlMap).length > 0) {
            try {
                const pkg = mod.parsePackage(xmlMap);
                const report = mod.buildReport(xmlMap, { rules: rulesConfigRef.current });
                setPackageData(mapToViewData(pkg, report));
            } catch (e) {
                console.error('[imf] parsePackage/buildReport error:', e);
                setParseError(String(e));
            }
        }
    }, []);

    const onDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setDragging(false);
        processFiles(e.dataTransfer.files);
    }, [processFiles]);

    const onDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setDragging(true);
    }, []);

    const onDragLeave = useCallback(() => setDragging(false), []);

    const onInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files) processFiles(e.target.files);
        e.target.value = '';
    }, [processFiles]);

    const removeFile = useCallback((uid: string) => {
        setFiles((prev) => prev.filter((f) => f.uid !== uid));
        setPackageData(null);
        setParseError(null);
        xmlMapRef.current = {};
    }, []);

    return (
        <section className="not-content" style={{ paddingBottom: 0 }}>
            <div style={{ marginBottom: '24px' }}>
                <h2 style={{ fontSize: '24px', fontWeight: 700, color: 'var(--sl-color-text, #dfdfd6)', margin: '0 0 8px' }}>
                    Try it in your browser
                </h2>
                <p style={{ fontSize: '14px', color: 'var(--sl-color-gray-3, #98989f)', margin: 0 }}>
                    Drop any IMF XML file — the parser runs entirely in WebAssembly. Nothing leaves your browser.
                </p>
            </div>

            {wasmError && (
                <div style={{
                    marginBottom: '16px',
                    padding: '10px 14px',
                    borderRadius: '8px',
                    background: 'rgba(248,113,113,0.1)',
                    border: '1px solid rgba(248,113,113,0.3)',
                    fontSize: '13px',
                    color: '#f87171',
                    fontFamily: 'monospace',
                }}>
                    Failed to load WASM: {wasmError}
                </div>
            )}

            <div
                onClick={() => wasmReady && inputRef.current?.click()}
                onDrop={onDrop}
                onDragOver={onDragOver}
                onDragLeave={onDragLeave}
                style={{
                    marginBottom: '16px',
                    padding: '36px 24px',
                    borderRadius: '12px',
                    border: `2px dashed ${dragging ? 'var(--sl-color-accent, #f97316)' : 'var(--sl-color-gray-5, #3c3f44)'}`,
                    background: dragging ? 'rgba(249,115,22,0.05)' : 'var(--hp-card-bg, #202127)',
                    cursor: wasmReady ? 'pointer' : 'default',
                    textAlign: 'center',
                    transition: 'border-color 0.2s, background 0.2s',
                    userSelect: 'none',
                }}
            >
                <input
                    ref={inputRef}
                    type="file"
                    accept=".xml"
                    multiple
                    onChange={onInputChange}
                    style={{ display: 'none' }}
                    disabled={!wasmReady}
                />

                {!wasmReady && !wasmError && (
                    <p style={{ fontSize: '13px', color: 'var(--sl-color-gray-4, #6a6a71)', margin: 0 }}>
                        Loading WebAssembly module…
                    </p>
                )}

                {wasmReady && (
                    <>
                        <div style={{ fontSize: '28px', marginBottom: '10px', opacity: dragging ? 1 : 0.35, transition: 'opacity 0.2s' }}>
                            ⬆
                        </div>
                        <p style={{ fontSize: '14px', color: 'var(--sl-color-text, #dfdfd6)', margin: '0 0 4px', fontWeight: 500 }}>
                            Drop ASSETMAP.xml, VOLINDEX.xml, PKL_*.xml, or CPL .xml files
                        </p>
                        <p style={{ fontSize: '12px', color: 'var(--sl-color-gray-4, #6a6a71)', margin: 0 }}>
                            or click to browse · max 2 MB per file · drop all files from a package to see the full view
                        </p>
                    </>
                )}
            </div>

            {/* Rules config panel */}
            {wasmReady && (
                <div style={{ marginBottom: '16px' }}>
                    <button
                        onClick={() => setShowRules(s => !s)}
                        style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '6px',
                            background: 'none',
                            border: 'none',
                            cursor: 'pointer',
                            color: showRules ? 'var(--sl-color-accent, #f97316)' : 'var(--sl-color-gray-4, #6a6a71)',
                            fontSize: '12px',
                            fontWeight: 500,
                            padding: '4px 0',
                            transition: 'color 0.15s',
                        }}
                    >
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
                            <circle cx="12" cy="12" r="3"/>
                        </svg>
                        Validation rules
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"
                            style={{ transform: showRules ? 'rotate(0deg)' : 'rotate(-90deg)', transition: 'transform 0.15s' }}>
                            <path d="m6 9 6 6 6-6"/>
                        </svg>
                    </button>

                    {showRules && (
                        <div style={{
                            marginTop: '8px',
                            padding: '12px 14px',
                            borderRadius: '10px',
                            border: '1px solid var(--sl-color-hairline, #2e2e32)',
                            background: 'var(--hp-card-bg, #202127)',
                            display: 'flex',
                            flexDirection: 'column',
                            gap: '10px',
                        }}>
                            {CONFIGURABLE_RULES.map(rule => {
                                const current = rulesConfig[rule.code] ?? 'error';
                                return (
                                    <div key={rule.code} style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
                                        <code style={{ fontSize: '11px', fontFamily: 'monospace', color: 'var(--sl-color-accent, #f97316)', flexShrink: 0 }}>{rule.code}</code>
                                        <span style={{ fontSize: '11px', color: 'var(--sl-color-gray-4, #6a6a71)', flex: 1, minWidth: '100px' }}>{rule.hint}</span>
                                        <select
                                            value={current}
                                            onChange={e => setRulesConfig(prev => ({ ...prev, [rule.code]: e.target.value as RuleSeverity }))}
                                            style={{
                                                background: 'var(--hp-code-bg, #2a2a30)',
                                                border: '1px solid var(--sl-color-gray-5, #3c3f44)',
                                                borderRadius: '6px',
                                                color: current === 'off' ? 'var(--sl-color-gray-4, #6a6a71)' : current === 'warn' ? '#f59e0b' : current === 'info' ? '#60a5fa' : current === 'critical' ? '#ef4444' : '#f87171',
                                                fontSize: '11px',
                                                fontWeight: 600,
                                                padding: '3px 8px',
                                                cursor: 'pointer',
                                                flexShrink: 0,
                                            }}
                                        >
                                            <option value="error">Error</option>
                                            <option value="warn">Warning</option>
                                            <option value="info">Info</option>
                                            <option value="off">Off</option>
                                            <option value="critical">Critical</option>
                                        </select>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            )}

            {files.length > 0 && (
                <>
                    {/* Parse error */}
                    {parseError && (
                        <div style={{
                            marginBottom: '16px',
                            padding: '10px 14px',
                            borderRadius: '8px',
                            background: 'rgba(248,113,113,0.1)',
                            border: '1px solid rgba(248,113,113,0.3)',
                            fontSize: '12px',
                            color: '#f87171',
                            fontFamily: 'monospace',
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-all',
                        }}>
                            {parseError}
                        </div>
                    )}

                    {/* Package view */}
                    {packageData && (
                        <div style={{ marginBottom: '20px' }}>
                            <IMFPackageViewer data={packageData} />
                        </div>
                    )}

                    {/* Individual file cards */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                        {files.map((f) => (
                            <FileCard key={f.uid} file={f} onRemove={() => removeFile(f.uid)} />
                        ))}
                    </div>
                </>
            )}
        </section>
    );
}
