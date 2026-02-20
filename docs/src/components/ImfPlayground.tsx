import { useState, useRef, useCallback, useEffect, useMemo } from 'react';

// ─── configurable rules ───────────────────────────────────────────────────────

type RuleSeverity = 'off' | 'info' | 'warn' | 'error' | 'critical';

const CONFIGURABLE_RULES: Array<{ code: string; label: string; hint: string }> = [
    { code: 'ST2067-2:2020:8.3/FileNotFound',        label: 'FileNotFound',      hint: 'Asset file not found on disk' },
    { code: 'ST2067-3:2020:7.2.2/SegmentDuration',   label: 'SegmentDuration',   hint: 'Unequal segment durations across tracks' },
];
import IMFPackageViewer, { type PackageViewData, type ValidationIssue } from './IMFPackageViewer';

// ─── global WASM handle ───────────────────────────────────────────────────────
// The WASM module is loaded by an `is:inline` <script> in index.astro using a
// native browser ES-module import (outside Vite's module graph). It exposes
// the parse functions on window.__imfWasm and fires 'imf-wasm-ready' when done.

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

type ParseState =
    | { tag: 'parsing' }
    | { tag: 'done'; result: unknown; sourceAsset?: unknown; validation?: unknown }
    | { tag: 'error'; message: string };

interface UploadedFile {
    uid: string;
    name: string;
    size: number;
    kind: FileKind | 'unknown';
    state: ParseState;
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

// ─── value coercion ───────────────────────────────────────────────────────────
// quick-xml serialises XML text+attribute elements as { "$text": "...", "@scope": "..." }.
// This helper extracts a renderable string from any field that may have that shape.
function asText(v: unknown): string {
    if (v == null) return '';
    if (typeof v === 'string') return v;
    if (typeof v === 'number' || typeof v === 'boolean') return String(v);
    if (typeof v === 'object') {
        const obj = v as Record<string, unknown>;
        if ('$text' in obj) return String(obj['$text'] ?? '');
        if ('text' in obj) return String(obj['text'] ?? '');
    }
    return String(v);
}

// ─── result renderers ─────────────────────────────────────────────────────────

function Row({ label, value }: { label: string; value: React.ReactNode }) {
    return (
        <div style={{
            display: 'flex',
            gap: '12px',
            padding: '6px 0',
            borderBottom: '1px solid #2e2e32',
            fontSize: '13px',
        }}>
            <span style={{ color: '#6a6a71', minWidth: '130px', flexShrink: 0 }}>{label}</span>
            <span style={{ color: '#dfdfd6', wordBreak: 'break-all' }}>{value}</span>
        </div>
    );
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function VolindexResult({ r }: { r: any }) {
    return <Row label="Volume index" value={String(r.Index)} />;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function AssetMapResult({ r }: { r: any }) {
    const assets: unknown[] = r.AssetList?.Asset ?? [];
    return (
        <div>
            <Row label="ID" value={asText(r.Id)} />
            <Row label="Issue date" value={asText(r.IssueDate)} />
            {r.VolumeCount !== undefined && <Row label="Volume count" value={String(r.VolumeCount)} />}
            {r.Issuer && <Row label="Issuer" value={asText(r.Issuer)} />}
            {r.Creator && <Row label="Creator" value={asText(r.Creator)} />}
            {r.AnnotationText && <Row label="Annotation" value={asText(r.AnnotationText)} />}
            <Row label="Assets" value={String(assets.length)} />
            {assets.length > 0 && (
                <div style={{ marginTop: '8px', display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
                    {assets.map((asset: any, i: number) => {
                        const path = asset.ChunkList?.Chunk?.[0]?.Path ?? '—';
                        const isPkl = asset.PackingList === true;
                        return (
                            <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: '8px', padding: '4px 0' }}>
                                <span style={{
                                    flexShrink: 0,
                                    fontSize: '10px',
                                    fontFamily: 'monospace',
                                    padding: '1px 6px',
                                    borderRadius: '4px',
                                    background: isPkl ? 'rgba(194,97,38,0.2)' : '#2a2a30',
                                    color: isPkl ? '#f97316' : '#6a6a71',
                                }}>
                                    {isPkl ? 'PKL' : 'ASSET'}
                                </span>
                                <div style={{ minWidth: 0 }}>
                                    <div style={{ fontSize: '11px', fontFamily: 'monospace', color: '#98989f', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{asset.Id}</div>
                                    <div style={{ fontSize: '11px', color: '#6a6a71', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{path}</div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function CplResult({ r }: { r: any }) {
    const locales: unknown[] = r.localeList?.locale ?? [];
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const langs: string[] = locales.flatMap((l: any) => l.languageList?.language ?? []);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const regions: string[] = locales.flatMap((l: any) => l.regionList?.region ?? []);
    const descCount: number = r.essenceDescriptorList?.essenceDescriptor?.length ?? 0;
    const segs: unknown[] = r.segmentList?.segment ?? [];
    const kindText = asText(r.contentKind);
    return (
        <div>
            <Row label="ID" value={asText(r.id)} />
            <Row label="Title" value={asText(r.contentTitle?.text) || asText(r.contentTitle) || '—'} />
            <Row label="Issue date" value={asText(r.issueDate)} />
            {kindText && <Row label="Content kind" value={kindText} />}
            {r.issuer && <Row label="Issuer" value={asText(r.issuer?.text) || asText(r.issuer)} />}
            {r.creator && <Row label="Creator" value={asText(r.creator?.text) || asText(r.creator)} />}
            {r.contentOriginator && <Row label="Content originator" value={asText(r.contentOriginator?.text) || asText(r.contentOriginator)} />}
            {langs.length > 0 && <Row label="Languages" value={langs.join(', ')} />}
            {regions.length > 0 && <Row label="Regions" value={regions.join(', ')} />}
            <Row label="Segments" value={String(segs.length)} />
            {descCount > 0 && <Row label="Essence descriptors" value={String(descCount)} />}
        </div>
    );
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function PklResult({ r }: { r: any }) {
    const assets: unknown[] = r.asset_list?.assets ?? [];
    return (
        <div>
            <Row label="ID" value={asText(r.id)} />
            <Row label="Issue date" value={asText(r.issue_date)} />
            {r.issuer && <Row label="Issuer" value={asText(r.issuer)} />}
            {r.creator && <Row label="Creator" value={asText(r.creator)} />}
            <Row label="Assets" value={String(assets.length)} />
            {assets.length > 0 && (
                <div style={{ marginTop: '8px', display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
                    {assets.map((asset: any, i: number) => (
                        <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: '8px', padding: '4px 0' }}>
                            <span style={{
                                flexShrink: 0,
                                fontSize: '10px',
                                fontFamily: 'monospace',
                                padding: '1px 6px',
                                borderRadius: '4px',
                                background: '#2a2a30',
                                color: '#6a6a71',
                            }}>
                                {String(asset.mime_type ?? 'ASSET')}
                            </span>
                            <div style={{ minWidth: 0 }}>
                                <div style={{ fontSize: '11px', fontFamily: 'monospace', color: '#98989f', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{asset.id}</div>
                                <div style={{ fontSize: '11px', color: '#6a6a71' }}>{formatBytes(Number(asset.size ?? 0))}</div>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
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
            border: '1px solid #2e2e32',
            background: '#202127',
            overflow: 'hidden',
        }}>
            <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
                padding: '10px 16px',
                borderBottom: '1px solid #2e2e32',
            }}>
                {file.kind !== 'unknown' ? (
                    <span style={{
                        fontSize: '11px',
                        fontFamily: 'monospace',
                        fontWeight: 600,
                        padding: '2px 8px',
                        borderRadius: '4px',
                        background: 'rgba(194,97,38,0.2)',
                        color: '#f97316',
                        flexShrink: 0,
                    }}>
                        {KIND_LABEL[file.kind]}
                    </span>
                ) : (
                    <span style={{ fontSize: '11px', color: '#6a6a71', fontStyle: 'italic', flexShrink: 0 }}>unknown</span>
                )}
                <span style={{ fontSize: '13px', color: '#dfdfd6', fontWeight: 500, flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {file.name}
                </span>
                <span style={{ fontSize: '11px', color: '#6a6a71', flexShrink: 0 }}>{formatBytes(file.size)}</span>
                <button
                    onClick={onRemove}
                    aria-label="Remove"
                    style={{
                        background: 'none',
                        border: 'none',
                        cursor: 'pointer',
                        color: '#6a6a71',
                        fontSize: '16px',
                        lineHeight: 1,
                        padding: '0 2px',
                        flexShrink: 0,
                    }}
                >×</button>
            </div>

            <div style={{ padding: '12px 16px' }}>
                {file.state.tag === 'parsing' && (
                    <span style={{ fontSize: '12px', color: '#98989f' }}>Parsing…</span>
                )}
                {file.state.tag === 'error' && (
                    <pre style={{
                        fontSize: '11px',
                        color: '#f87171',
                        fontFamily: 'monospace',
                        margin: 0,
                        whiteSpace: 'pre-wrap',
                        wordBreak: 'break-all',
                    }}>
                        {file.state.message}
                    </pre>
                )}
                {file.state.tag === 'done' && file.kind === 'volindex' && (
                    <VolindexResult r={file.state.result} />
                )}
                {file.state.tag === 'done' && file.kind === 'assetmap' && (
                    <AssetMapResult r={file.state.result} />
                )}
                {file.state.tag === 'done' && file.kind === 'cpl' && (
                    <CplResult r={file.state.result} />
                )}
                {file.state.tag === 'done' && file.kind === 'pkl' && file.state.result != null && (
                    <PklResult r={file.state.result} />
                )}
                {file.state.tag === 'done' && file.kind === 'opl' && (
                    <span style={{ fontSize: '12px', color: '#6a6a71' }}>
                        OPL (Output Profile List) is a DCP-only document — not part of IMF. No parsing needed.
                    </span>
                )}
                {file.state.tag === 'done' && file.kind === 'unknown' && (
                    <span style={{ fontSize: '12px', color: '#6a6a71' }}>
                        File name not recognized. Expected <code>VOLINDEX.xml</code>, <code>ASSETMAP.xml</code>, <code>PKL_*.xml</code>, or a CPL <code>.xml</code> file.
                    </span>
                )}
            </div>
        </div>
    );
}

// ─── package data assembly ────────────────────────────────────────────────────

// Minimal source asset for CPLs where extractSourceAsset fails.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function makeFallbackSourceAsset(cpl: any): Record<string, unknown> {
    return {
        contentKind: asText(cpl?.contentKind) || 'UNKNOWN',
        contentTitle: asText(cpl?.contentTitle?.text) || asText(cpl?.contentTitle) || null,
        territory: null,
        editRate: null,
        frameRate: null,
        duration: null,
        audioLanguages: [],
        subtitleLanguages: [],
        captionLanguages: [],
        forcedNarrativeLanguages: [],
        audioType: null,
        videoQuality: null,
        videoDynamicRange: null,
        tracks: { AUDIO: [], VIDEO: [], SUBTITLES: [], CAPTIONS: [], FORCED_NARRATIVE: [] },
        sequences: [],
    };
}

function buildPackageData(files: UploadedFile[]): PackageViewData | null {
    const assetmapFile = files.find(f => f.kind === 'assetmap' && f.state.tag === 'done');
    // Include all successfully parsed CPLs regardless of whether sourceAsset extraction succeeded.
    const cplFiles = files.filter(f => f.kind === 'cpl' && f.state.tag === 'done');
    if (!assetmapFile || cplFiles.length === 0) return null;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const am = (assetmapFile.state as any).result as any;
    const assets: unknown[] = am?.AssetList?.Asset ?? [];
    const pklFiles = files.filter(f => f.kind === 'pkl');

    const pkg: PackageViewData['package'] = {
        assetMapId: asText(am?.Id) || 'urn:uuid:unknown',
        volumeIndex: 1,
        assetCount: assets.length,
        cplCount: cplFiles.length,
        pklCount: pklFiles.length || assets.filter((a: unknown) => (a as Record<string, unknown>)?.PackingList === true).length,
    };

    const cpls: PackageViewData['cpls'] = cplFiles.map(f => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const state = f.state as any;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const cpl = state.result as any;
        const segs: unknown[] = cpl?.segmentList?.segment ?? [];
        // Use extracted source asset when available, fall back to minimal stub otherwise.
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const sourceAsset = (state.sourceAsset as any) ?? makeFallbackSourceAsset(cpl);
        return {
            id: String(cpl?.id ?? f.uid),
            title: asText(cpl?.contentTitle?.text) || asText(cpl?.contentTitle) || f.name,
            issuer: asText(cpl?.issuer?.text) || asText(cpl?.issuer) || null,
            creator: asText(cpl?.creator?.text) || asText(cpl?.creator) || null,
            issueDate: asText(cpl?.issueDate) || null,
            applicationProfile: null,
            segmentCount: segs.length,
            timecodeStart: null,
            isSupplemental: false,
            unresolvedAncestorAssetIds: [],
            sourceAsset,
            markers: [],
            deliveryComparison: null,
        };
    });

    // Aggregate validation issues from all CPLs.
    // validateCplWithSpecSelection returns a ValidationReport with severity buckets.
    const allIssues: ValidationIssue[] = cplFiles.flatMap(f => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const val = (f.state as any).validation as any;
        if (!val) return [];
        // Flatten ValidationReport buckets into a single list.
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const flat: any[] = [
            ...(val.critical ?? []),
            ...(val.errors ?? []),
            ...(val.warnings ?? []),
            ...(val.info ?? []),
        ];
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        return flat.map((issue: any) => ({
            severity: String(issue.severity ?? ''),
            category: String(issue.category ?? ''),
            code: String(issue.code ?? ''),
            message: String(issue.message ?? ''),
            suggestion: issue.suggestion ?? null,
            source: undefined,
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            cplId: ((f.state as any).result as any)?.id,
        }));
    });
    const hasErrors = allIssues.some(i => i.severity === 'Error' || i.severity === 'Critical');
    const hasWarnings = allIssues.some(i => i.severity === 'Warning');
    // Only show a status if at least one CPL was actually validated.
    const anyValidated = cplFiles.some(f => (f.state as any).validation != null);
    const status = !anyValidated ? null : hasErrors ? 'Invalid' : hasWarnings ? 'ValidWithWarnings' : 'Valid';

    return { package: pkg, cpls, validation: { status: status as PackageViewData['validation']['status'], issues: allIssues } };
}

// ─── main component ───────────────────────────────────────────────────────────

export default function ImfPlayground() {
    const [files, setFiles] = useState<UploadedFile[]>([]);
    const [wasmReady, setWasmReady] = useState(false);
    const [wasmError, setWasmError] = useState<string | null>(null);
    const [dragging, setDragging] = useState(false);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const [packageValidation, setPackageValidation] = useState<any>(null);
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
        if (!mod || !Object.keys(xmlMapRef.current).some(k => k.toUpperCase() === 'ASSETMAP.XML')) return;
        try {
            const result = mod.validatePackage(xmlMapRef.current, rulesConfig);
            setPackageValidation(result);
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
            state: { tag: 'parsing' },
        }));
        setFiles(entries);
        setPackageValidation(null);

        const xmlMap: Record<string, string> = {};
        xmlMapRef.current = {};

        for (let i = 0; i < list.length; i++) {
            const f = list[i];
            const entry = entries[i];
            let state: ParseState;
            try {
                const xml = await f.text();
                const mod = getWasmModule();
                if (!mod) throw new Error('WASM module not ready');

                // Collect all XML for package-level validation
                if (entry.kind !== 'unknown') {
                    xmlMap[f.name] = xml;
                    xmlMapRef.current = { ...xmlMap };
                }

                if (entry.kind === 'volindex') {
                    state = { tag: 'done', result: mod.parseVolindexTyped(xml) };
                } else if (entry.kind === 'assetmap') {
                    state = { tag: 'done', result: mod.parseAssetmapTyped(xml) };
                } else if (entry.kind === 'pkl') {
                    state = { tag: 'done', result: mod.parsePklTyped(xml) };
                } else if (entry.kind === 'opl') {
                    state = { tag: 'done', result: null };
                } else if (entry.kind === 'cpl') {
                    const result = mod.parseCplTyped(xml);
                    let sourceAsset: unknown = null;
                    let validation: unknown = null;
                    try { sourceAsset = mod.extractSourceAsset(xml); } catch { /* non-critical */ }
                    try { validation = mod.validateCplWithSpecSelection(xml, 'auto', 'auto'); } catch { /* non-critical */ }
                    state = { tag: 'done', result, sourceAsset, validation };
                } else {
                    state = { tag: 'done', result: null };
                }
            } catch (e: unknown) {
                state = { tag: 'error', message: String(e) };
            }
            setFiles((prev) =>
                prev.map((p) => (p.uid === entry.uid ? { ...p, state } : p)),
            );
        }

        // Package-level validation — runs after all files are parsed
        const mod = getWasmModule();
        if (mod && Object.keys(xmlMap).some(k => k.toUpperCase() === 'ASSETMAP.XML')) {
            try {
                const result = mod.validatePackage(xmlMap, rulesConfigRef.current);
                setPackageValidation(result);
            } catch (e) {
                console.error('[imf] validatePackage error:', e);
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
        setPackageValidation(null);
        xmlMapRef.current = {};
    }, []);

    // Assemble package data once assetmap + >=1 CPL with source asset are ready.
    // Merge package-level validation from ValidationReport (severity buckets) when available.
    const packageData = useMemo(() => {
        const base = buildPackageData(files);
        if (!base) return null;
        if (!packageValidation) return base;

        // validatePackage returns a ValidationReport with severity buckets.
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const pv = packageValidation as any;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const flatIssues: any[] = [
            ...(pv.critical ?? []),
            ...(pv.errors ?? []),
            ...(pv.warnings ?? []),
            ...(pv.info ?? []),
        ];

        const hasCritical = (pv.critical?.length ?? 0) > 0;
        const hasPkgErrors = (pv.errors?.length ?? 0) > 0;
        const hasPkgWarnings = (pv.warnings?.length ?? 0) > 0;
        const pkgStatus = hasCritical || hasPkgErrors ? 'Invalid' : hasPkgWarnings ? 'ValidWithWarnings' : 'Valid';

        const summary = {
            total: flatIssues.length,
            critical: pv.critical?.length ?? 0,
            errors: pv.errors?.length ?? 0,
            warnings: pv.warnings?.length ?? 0,
            info: pv.info?.length ?? 0,
            is_playable: pv.is_playable ?? true,
            is_compliant: pv.is_compliant ?? true,
        };

        return {
            ...base,
            validation: {
                status: pkgStatus as typeof base.validation.status,
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                issues: flatIssues.map((issue: any) => ({
                    severity: String(issue.severity ?? ''),
                    category: String(issue.category ?? ''),
                    code: String(issue.code ?? ''),
                    message: String(issue.message ?? ''),
                    suggestion: issue.suggestion ?? null,
                    source: undefined,
                    cplId: issue.location?.cpl_id ?? undefined,
                })),
                summary,
            },
        };
    }, [files, packageValidation]);

    return (
        <section style={{ paddingBottom: '80px' }}>
            <div style={{ marginBottom: '24px' }}>
                <h2 style={{ fontSize: '24px', fontWeight: 700, color: '#dfdfd6', margin: '0 0 8px' }}>
                    Try it in your browser
                </h2>
                <p style={{ fontSize: '14px', color: '#98989f', margin: 0 }}>
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
                    border: `2px dashed ${dragging ? '#f97316' : '#3c3f44'}`,
                    background: dragging ? 'rgba(249,115,22,0.05)' : '#202127',
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
                    <p style={{ fontSize: '13px', color: '#6a6a71', margin: 0 }}>
                        Loading WebAssembly module…
                    </p>
                )}

                {wasmReady && (
                    <>
                        <div style={{ fontSize: '28px', marginBottom: '10px', opacity: dragging ? 1 : 0.35, transition: 'opacity 0.2s' }}>
                            ⬆
                        </div>
                        <p style={{ fontSize: '14px', color: '#dfdfd6', margin: '0 0 4px', fontWeight: 500 }}>
                            Drop ASSETMAP.xml, VOLINDEX.xml, PKL_*.xml, or CPL .xml files
                        </p>
                        <p style={{ fontSize: '12px', color: '#6a6a71', margin: 0 }}>
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
                            color: showRules ? '#f97316' : '#6a6a71',
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
                            border: '1px solid #2e2e32',
                            background: '#202127',
                            display: 'flex',
                            flexDirection: 'column',
                            gap: '10px',
                        }}>
                            {CONFIGURABLE_RULES.map(rule => {
                                const current = rulesConfig[rule.code] ?? 'error';
                                return (
                                    <div key={rule.code} style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
                                        <code style={{ fontSize: '11px', fontFamily: 'monospace', color: '#f97316', flexShrink: 0 }}>{rule.code}</code>
                                        <span style={{ fontSize: '11px', color: '#6a6a71', flex: 1, minWidth: '100px' }}>{rule.hint}</span>
                                        <select
                                            value={current}
                                            onChange={e => setRulesConfig(prev => ({ ...prev, [rule.code]: e.target.value as RuleSeverity }))}
                                            style={{
                                                background: '#2a2a30',
                                                border: '1px solid #3c3f44',
                                                borderRadius: '6px',
                                                color: current === 'off' ? '#6a6a71' : current === 'warn' ? '#f59e0b' : current === 'info' ? '#60a5fa' : current === 'critical' ? '#ef4444' : '#f87171',
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
                    {/* Package view: shown when assetmap + >=1 CPL are ready */}
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
