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

// ─── map validate() result → IMFPackageViewer data ───────────────────────────
//
// validate() returns { package: Imferno, validation: ValidationReport }
// In WASM builds, CPL fields are camelCase (feature = "wasm" serde rename).
// Imferno top-level fields are camelCase (serde rename_all = "camelCase").

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function langFromDescriptor(ed: any): string | null {
    const wave = ed?.wavePcmDescriptor;
    const sf = wave?.subDescriptors?.soundfieldGroupLabelSubDescriptor;
    if (sf?.rfc5646SpokenLanguage) return String(sf.rfc5646SpokenLanguage);
    const tt = ed?.dcTimedTextDescriptor;
    const ttLangs = tt?.rfc5646LanguageTagList;
    if (Array.isArray(ttLangs) && ttLangs.length > 0) return String(ttLangs[0]);
    const iab = ed?.iabEssenceDescriptor;
    const iabSf = iab?.subDescriptors?.iabSoundfieldLabelSubDescriptor;
    if (iabSf?.rfc5646SpokenLanguage) return String(iabSf.rfc5646SpokenLanguage);
    return null;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function channelsFromDescriptor(ed: any): number | null {
    return ed?.wavePcmDescriptor?.channelCount ?? ed?.iabEssenceDescriptor?.channelCount ?? null;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function soundfieldFromDescriptor(ed: any): string | null {
    const sf = ed?.wavePcmDescriptor?.subDescriptors?.soundfieldGroupLabelSubDescriptor;
    if (sf?.mcaTagSymbol) return String(sf.mcaTagSymbol);
    if (sf?.mcaTagName) return String(sf.mcaTagName);
    if (ed?.iabEssenceDescriptor) return 'Atmos';
    return null;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mapValidateResult(result: any): any {
    console.log('[imf] validate() raw result:', JSON.stringify(result).slice(0, 2000));
    const pkg = result.package;
    const v = result.validation;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const flatIssues: any[] = [
        ...(v.critical ?? []),
        ...(v.errors ?? []),
        ...(v.warnings ?? []),
        ...(v.info ?? []),
    ];

    const valid = !(v.critical?.length > 0) && !(v.errors?.length > 0);

    // Build descriptor map per CPL: id → EssenceDescriptor
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function descriptorMap(cpl: any): Record<string, any> {
        const m: Record<string, any> = {}; // eslint-disable-line @typescript-eslint/no-explicit-any
        for (const ed of cpl?.essenceDescriptorList?.essenceDescriptor ?? []) {
            if (ed.id) m[ed.id] = ed;
        }
        return m;
    }

    const cplEntries = Object.entries(pkg?.compositionPlaylists ?? {});

    return {
        package: {
            assetMapId: pkg?.assetMap?.id ?? '',
            volumeIndex: pkg?.volumeIndex?.index ?? 1,
            assetCount: pkg?.assetMap?.assetList?.assets?.length ?? 0,
            cplCount: cplEntries.length,
            pklCount: Object.keys(pkg?.packingLists ?? {}).length,
        },
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        cpls: cplEntries.map(([, cpl]: [string, any]) => {
            console.log('[imf] CPL keys:', Object.keys(cpl));
            console.log('[imf] CPL.contentTitle:', cpl.contentTitle, cpl.ContentTitle);
            console.log('[imf] CPL.segmentList:', cpl.segmentList, cpl.SegmentList);
            if (cpl.segmentList) console.log('[imf] segmentList keys:', Object.keys(cpl.segmentList));
            if (cpl.SegmentList) console.log('[imf] SegmentList keys:', Object.keys(cpl.SegmentList));
            const er = cpl.editRate ?? cpl.EditRate;
            const editRate = er ? `${er.numerator ?? er.Numerator} ${er.denominator ?? er.Denominator}` : null;
            const descs = descriptorMap(cpl);
            const rawTitle = cpl.contentTitle ?? cpl.ContentTitle;
            const title = typeof rawTitle === 'string' ? rawTitle : rawTitle?.text ?? rawTitle?.Text ?? '';
            const rawSegList = cpl.segmentList ?? cpl.SegmentList;
            const segments = rawSegList?.segments ?? rawSegList?.Segments ?? rawSegList?.segment ?? rawSegList?.Segment ?? [];
            if (!Array.isArray(segments)) console.log('[imf] segments is not array:', segments);
            const contentKind = typeof cpl.contentKind === 'string' ? cpl.contentKind : cpl.contentKind?.value ?? null;

            // Flatten sequences across segments, merge by trackId
            // Sequence type keys → IMFPackageViewer type names
            const seqTypeKeys: [string, string][] = [
                ['mainImageSequences', 'MainImageSequence'],
                ['mainAudioSequences', 'MainAudioSequence'],
                ['subtitlesSequences', 'SubtitlesSequence'],
                ['hearingImpairedCaptionsSequences', 'HearingImpairedCaptionsSequence'],
                ['forcedNarrativeSequences', 'ForcedNarrativeSequence'],
                ['iabSequences', 'IABSequence'],
                ['isxdSequences', 'ISXDSequence'],
            ];

            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const sequences: any[] = [];
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            for (const seg of segments) {
                const sl = seg.sequenceList ?? {};
                for (const [key, typeName] of seqTypeKeys) {
                    const arr = sl[key];
                    if (!Array.isArray(arr)) continue;
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    for (const seq of arr) {
                        const trackId = seq.trackId ?? '';
                        const resources = (seq.resourceList?.resource ?? []).map((r: any) => ({ // eslint-disable-line @typescript-eslint/no-explicit-any
                            id: r.id ?? '',
                            intrinsicDuration: r.intrinsicDuration ?? 0,
                            sourceDuration: r.sourceDuration ?? r.intrinsicDuration ?? 0,
                            sourceEncoding: r.sourceEncoding ?? null,
                            trackFileId: r.trackFileId ?? null,
                            editRate: r.editRate ? `${r.editRate.numerator} ${r.editRate.denominator}` : editRate,
                            entryPoint: r.entryPoint ?? null,
                        }));
                        const existing = sequences.find((s: any) => s.trackId === trackId); // eslint-disable-line @typescript-eslint/no-explicit-any
                        if (existing) {
                            existing.sequenceResources.push(...resources);
                        } else {
                            const seUuid = (seq.resourceList?.resource ?? [])[0]?.sourceEncoding;
                            const ed = seUuid ? descs[seUuid] : null;
                            sequences.push({
                                type: typeName,
                                id: seq.id ?? '',
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

            // Markers
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const markers: any[] = [];
            for (const seg of segments) {
                for (const ms of seg.sequenceList?.markerSequences ?? []) {
                    for (const r of ms.resourceList?.resources ?? []) {
                        for (const m of r.markers ?? []) {
                            markers.push({
                                label: typeof m.label === 'string' ? m.label : m.label?.value ?? '',
                                offset: m.offset ?? null,
                                scope: m.label?.scope ?? null,
                            });
                        }
                    }
                }
            }

            return {
                id: cpl.id ?? '',
                title,
                applicationProfile: (() => {
                    for (const ext of cpl.extensionProperties?.extensionProperties ?? []) {
                        const appId = ext.applicationIdentification;
                        if (Array.isArray(appId) && appId.length > 0) return appId[0];
                        if (typeof appId === 'string') return appId;
                    }
                    return null;
                })(),
                segmentCount: segments.length,
                timecodeStart: cpl.compositionTimecode?.timecodeStartAddress ?? null,
                isSupplemental: false,
                markers,
                sourceAsset: {
                    contentKind,
                    contentTitle: title,
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
            const result = mod.validate(xmlMapRef.current, { rules: rulesConfig });
            setPackageData(mapValidateResult(result));
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
                const result = mod.validate(xmlMap, { rules: rulesConfigRef.current });
                setPackageData(mapValidateResult(result));
            } catch (e) {
                console.error('[imf] validate error:', e);
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
