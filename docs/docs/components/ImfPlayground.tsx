import { useState, useRef, useCallback, useEffect } from 'react';

// ─── configurable rules ───────────────────────────────────────────────────────

type RuleSeverity = 'off' | 'info' | 'warn' | 'error' | 'critical';

// Segmented-control options, mildest first. Colours match the viewer's
// issue list (red = error/critical, amber = warning, zinc = info/off).
const SEVERITY_OPTIONS: Array<{ value: RuleSeverity; label: string; activeClass: string }> = [
    { value: 'off', label: 'Off', activeClass: 'text-zinc-500 dark:text-zinc-400' },
    { value: 'info', label: 'Info', activeClass: 'text-blue-500' },
    { value: 'warn', label: 'Warning', activeClass: 'text-amber-600' },
    { value: 'error', label: 'Error', activeClass: 'text-red-600' },
    { value: 'critical', label: 'Critical', activeClass: 'text-red-700 dark:text-red-500' },
];

const CONFIGURABLE_RULES: Array<{ code: string; label: string; hint: string }> = [
    { code: 'ST2067-2:2020:8.3/FileNotFound',        label: 'FileNotFound',      hint: 'Asset file not found on disk' },
    { code: 'ST2067-3:2020:7.2.2/SegmentDuration',   label: 'SegmentDuration',   hint: 'Unequal segment durations across tracks' },
];
import IMFPackageViewer from './IMFPackageViewer';
import PackageFileBrowser, { type BrowserFile } from './PackageFileBrowser';
import { base } from './base';

// ─── sample packages (docs/public/samples/<id>/) ─────────────────────────────

const SAMPLES: Array<{ id: string; label: string; year: string; tags: string[] }> = [
    { id: 'sol-levante', label: 'Sol Levante', year: '2020', tags: ['UHD', '24 fps', 'Dolby Vision', '2× JA audio'] },
    { id: 'meridian', label: 'Meridian', year: '2016', tags: ['UHD', '59.94 fps', '5.1 audio', 'Timed text'] },
];

interface SampleIndex {
    id: string;
    title: string;
    description: string;
    attribution: string;
    attributionUrl: string;
    files: string[];
}

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

function makeUid() {
    return Math.random().toString(36).slice(2);
}

// ─── map validate() result → IMFPackageViewer data ───────────────────────────
//
// validate() returns { package: Imferno, validation: ValidationReport }
// In WASM builds, CPL fields are camelCase (feature = "wasm" serde rename).
// Imferno top-level fields are camelCase (serde rename_all = "camelCase").

// eslint-disable-next-line @typescript-eslint/no-explicit-any
// Helper: try both PascalCase (actual JSON) and camelCase (fallback) keys
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function pick(obj: any, ...keys: string[]): any {
    if (!obj) return null;
    for (const k of keys) {
        if (obj[k] != null) return obj[k];
    }
    return null;
}

function langFromDescriptor(ed: any): string | null {
    const wave = pick(ed, 'wavePCMDescriptor', 'wavePcmDescriptor');
    const waveSubs = pick(wave, 'SubDescriptors', 'subDescriptors');
    const sf = pick(waveSubs, 'SoundfieldGroupLabelSubDescriptor', 'soundfieldGroupLabelSubDescriptor');
    const sfLang = pick(sf, 'RFC5646SpokenLanguage', 'rfc5646SpokenLanguage');
    if (sfLang) return String(sfLang);
    const tt = ed?.dcTimedTextDescriptor;
    const ttLangs = pick(tt, 'RFC5646LanguageTagList', 'rfc5646LanguageTagList');
    if (Array.isArray(ttLangs) && ttLangs.length > 0) return String(ttLangs[0]);
    const iab = ed?.iabEssenceDescriptor;
    const iabSubs = pick(iab, 'SubDescriptors', 'subDescriptors');
    const iabSf = pick(iabSubs, 'IABSoundfieldLabelSubDescriptor', 'iabSoundfieldLabelSubDescriptor');
    const iabLang = pick(iabSf, 'RFC5646SpokenLanguage', 'rfc5646SpokenLanguage');
    if (iabLang) return String(iabLang);
    return null;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function channelsFromDescriptor(ed: any): number | null {
    const wave = pick(ed, 'wavePCMDescriptor', 'wavePcmDescriptor');
    const waveCh = pick(wave, 'ChannelCount', 'channelCount');
    if (waveCh != null) return Number(waveCh);
    const iabCh = pick(ed?.iabEssenceDescriptor, 'ChannelCount', 'channelCount');
    if (iabCh != null) return Number(iabCh);
    return null;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function soundfieldFromDescriptor(ed: any): string | null {
    const wave = pick(ed, 'wavePCMDescriptor', 'wavePcmDescriptor');
    const waveSubs = pick(wave, 'SubDescriptors', 'subDescriptors');
    const sf = pick(waveSubs, 'SoundfieldGroupLabelSubDescriptor', 'soundfieldGroupLabelSubDescriptor');
    const sym = pick(sf, 'MCATagSymbol', 'mcaTagSymbol');
    if (sym) return String(sym);
    const name = pick(sf, 'MCATagName', 'mcaTagName');
    if (name) return String(name);
    if (ed?.iabEssenceDescriptor) return 'Atmos';
    return null;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
// Helper: get a field trying camelCase, snake_case, and PascalCase
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function g(obj: any, ...keys: string[]): any {
    if (!obj) return undefined;
    for (const k of keys) {
        if (obj[k] !== undefined) return obj[k];
    }
    return undefined;
}

function mapValidateResult(result: any): any {
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
        const edl = g(cpl, 'essenceDescriptorList', 'EssenceDescriptorList');
        for (const ed of g(edl, 'essenceDescriptors', 'essenceDescriptor', 'EssenceDescriptor') ?? []) {
            const id = g(ed, 'id', 'Id');
            if (id) m[id] = ed;
        }
        return m;
    }

    const cplEntries = Object.entries(pkg?.compositionPlaylists ?? {});

    return {
        package: {
            assetMapId: g(g(pkg, 'assetMap', 'asset_map'), 'id') ?? '',
            volumeIndex: g(g(pkg, 'volumeIndex', 'volume_index'), 'index', 'Index') ?? 1,
            assetCount: g(g(g(pkg, 'assetMap', 'asset_map'), 'assetList', 'asset_list'), 'assets')?.length ?? 0,
            cplCount: cplEntries.length,
            pklCount: Object.keys(g(pkg, 'packingLists', 'packing_lists') ?? {}).length,
        },
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        cpls: cplEntries.map(([, cpl]: [string, any]) => {
            const er = cpl.editRate;
            const editRate = er ? `${er.numerator} ${er.denominator}` : null;
            const descs = descriptorMap(cpl);
            const rawTitle = cpl.contentTitle ?? cpl.ContentTitle;
            const title = typeof rawTitle === 'string' ? rawTitle : rawTitle?.['$text'] ?? rawTitle?.text ?? '';
            const rawSegList = cpl.segmentList ?? cpl.SegmentList;
            const segments = rawSegList?.segments ?? rawSegList?.Segments ?? rawSegList?.segment ?? rawSegList?.Segment ?? [];
            if (!Array.isArray(segments)) return null; // skip malformed CPLs
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
                        const resources = (seq.resourceList?.resources ?? []).map((r: any) => ({ // eslint-disable-line @typescript-eslint/no-explicit-any
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
                            const seUuid = (seq.resourceList?.resources ?? [])[0]?.sourceEncoding;
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

// ─── main component ───────────────────────────────────────────────────────────

export default function ImfPlayground() {
    const [files, setFiles] = useState<UploadedFile[]>([]);
    const [wasmReady, setWasmReady] = useState(false);
    const [wasmError, setWasmError] = useState<string | null>(null);
    const [dragging, setDragging] = useState(false);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const [packageData, setPackageData] = useState<any | null>(null);
    const [parseError, setParseError] = useState<string | null>(null);
    const [skippedCount, setSkippedCount] = useState<number>(0);
    const [rulesConfig, setRulesConfig] = useState<Record<string, RuleSeverity>>({
        'ST2067-2:2020:8.3/FileNotFound': 'info',
    });
    const rulesConfigRef = useRef<Record<string, RuleSeverity>>({});
    rulesConfigRef.current = rulesConfig;
    const [showRules, setShowRules] = useState(false);
    const xmlMapRef = useRef<Record<string, string>>({});
    const [browserFiles, setBrowserFiles] = useState<BrowserFile[]>([]);
    const [sample, setSample] = useState<SampleIndex | null>(null);
    const [loadingSample, setLoadingSample] = useState<string | null>(null);
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
        const all = Array.from(fileList);
        const list = all.filter(
            (f) => f.name.toLowerCase().endsWith('.xml') && f.size <= MAX_BYTES,
        );
        const skipped = all.filter(
            (f) => !f.name.toLowerCase().endsWith('.xml') && !f.name.startsWith('.'),
        );
        setSkippedCount(skipped.length);
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
        setBrowserFiles(Object.entries(xmlMap).map(([name, text]) => ({ name, text })));

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
        setSample(null);
        processFiles(e.dataTransfer.files);
    }, [processFiles]);

    // Fetch a bundled sample package and run it through the same path as
    // dropped files.
    const loadSample = useCallback(async (id: string) => {
        setLoadingSample(id);
        setParseError(null);
        try {
            const dir = `${base}samples/${id}/`;
            const indexRes = await fetch(`${dir}index.json`);
            if (!indexRes.ok) throw new Error(`index.json: HTTP ${indexRes.status}`);
            const index: SampleIndex = await indexRes.json();
            const fetched = await Promise.all(
                index.files.map(async (name) => {
                    const res = await fetch(dir + name);
                    if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
                    return new File([await res.text()], name, { type: 'application/xml' });
                }),
            );
            setSample(index);
            await processFiles(fetched);
        } catch (e) {
            setParseError(`Could not load sample package: ${e}`);
        } finally {
            setLoadingSample(null);
        }
    }, [processFiles]);

    const onDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setDragging(true);
    }, []);

    const onDragLeave = useCallback(() => setDragging(false), []);

    const onInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        setSample(null);
        if (e.target.files) processFiles(e.target.files);
        e.target.value = '';
    }, [processFiles]);

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

            {/* Sample packages */}
            <div className="mb-4">
                <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 dark:text-zinc-500 mb-2">
                    Or try a public package
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {SAMPLES.map(({ id, label, year, tags }) => {
                        const isLoading = loadingSample === id;
                        const isLoaded = sample?.id === id && !isLoading;
                        return (
                            <button
                                key={id}
                                type="button"
                                disabled={!wasmReady || loadingSample !== null}
                                onClick={() => loadSample(id)}
                                className={`group flex items-center gap-3 px-4 py-3 text-left rounded-xl border bg-white dark:bg-zinc-900 transition-all cursor-pointer disabled:cursor-default ${
                                    isLoaded
                                        ? 'border-amber-600/60 ring-1 ring-amber-600/20'
                                        : 'border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600'
                                }`}
                            >
                                <span className="w-8 h-8 flex-shrink-0 rounded-lg bg-amber-600/10 text-amber-600 flex items-center justify-center">
                                    <svg width={16} height={16} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
                                        <rect width="18" height="18" x="3" y="3" rx="2" /><path d="M7 3v18" /><path d="M3 7.5h4" /><path d="M3 12h18" /><path d="M3 16.5h4" /><path d="M17 3v18" /><path d="M17 7.5h4" /><path d="M17 16.5h4" />
                                    </svg>
                                </span>
                                <span className="flex-1 min-w-0">
                                    <span className="flex items-baseline gap-1.5">
                                        <span className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">{label}</span>
                                        <span className="text-[11px] text-zinc-400 dark:text-zinc-500">Netflix · {year}</span>
                                    </span>
                                    <span className="mt-1 flex flex-wrap gap-1">
                                        {tags.map(t => (
                                            <span key={t} className="inline-flex items-center px-1.5 rounded-md text-[10px] font-medium leading-[18px] bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400">{t}</span>
                                        ))}
                                    </span>
                                </span>
                                <span className={`flex-shrink-0 text-[11px] font-medium transition-colors ${isLoaded ? 'text-amber-600' : 'text-zinc-400 dark:text-zinc-500 group-hover:text-amber-600'}`}>
                                    {isLoading ? 'Loading…' : isLoaded ? 'Loaded' : 'Load →'}
                                </span>
                            </button>
                        );
                    })}
                </div>
                <p className="mt-2 mb-0 text-[11px] text-zinc-400 dark:text-zinc-500">
                    XML manifests only (no MXF essence). © Netflix, CC BY 4.0, via{' '}
                    <a href="https://opencontent.netflix.com/" target="_blank" rel="noreferrer" className="underline decoration-zinc-300 dark:decoration-zinc-600 hover:text-amber-600">Netflix Open Content</a>.
                </p>
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
                        <div className="mt-2 border border-zinc-200 dark:border-zinc-700 rounded-xl bg-white dark:bg-zinc-900 divide-y divide-zinc-100 dark:divide-zinc-800">
                            {CONFIGURABLE_RULES.map(rule => {
                                const current = rulesConfig[rule.code] ?? 'error';
                                return (
                                    <div key={rule.code} className="flex items-center gap-3 flex-wrap px-4 py-3">
                                        <div className="flex-1 min-w-[200px]">
                                            <code className="block text-[11px] font-mono text-zinc-700 dark:text-zinc-300">{rule.code}</code>
                                            <span className="block text-[11px] text-zinc-400 dark:text-zinc-500 mt-0.5">{rule.hint}</span>
                                        </div>
                                        <div className="flex gap-1 bg-zinc-100 dark:bg-zinc-800 rounded-lg p-1" role="radiogroup" aria-label={`Severity for ${rule.label}`}>
                                            {SEVERITY_OPTIONS.map(opt => {
                                                const active = current === opt.value;
                                                return (
                                                    <button
                                                        key={opt.value}
                                                        type="button"
                                                        role="radio"
                                                        aria-checked={active}
                                                        onClick={() => setRulesConfig(prev => ({ ...prev, [rule.code]: opt.value }))}
                                                        className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
                                                            active
                                                                ? `bg-white dark:bg-zinc-900 shadow-sm ${opt.activeClass}`
                                                                : 'text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200'
                                                        }`}
                                                    >
                                                        {opt.label}
                                                    </button>
                                                );
                                            })}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            )}

            {files.length > 0 && (
                <>
                    {/* Skipped non-XML files notice */}
                    {skippedCount > 0 && (
                        <div style={{
                            marginBottom: '12px',
                            padding: '8px 12px',
                            borderRadius: '8px',
                            background: 'rgba(148,163,184,0.1)',
                            border: '1px solid rgba(148,163,184,0.2)',
                            fontSize: '12px',
                            color: '#94a3b8',
                        }}>
                            Ignored {skippedCount} non-XML file{skippedCount === 1 ? '' : 's'} (MXF essence is not needed for validation; only manifest XMLs are read).
                        </div>
                    )}

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
                            <IMFPackageViewer data={packageData}>
                                {/* Raw manifests, as a section of the package card */}
                                <PackageFileBrowser files={browserFiles} />
                            </IMFPackageViewer>
                        </div>
                    )}
                </>
            )}
        </section>
    );
}
