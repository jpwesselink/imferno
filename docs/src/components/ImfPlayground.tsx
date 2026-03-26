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

/** Convert edit rate from "N/D" (Rust) to "N D" (component expects whitespace-split) */
function normalizeEditRate(er: string | null | undefined): string | null {
    if (!er) return null;
    return er.replace('/', ' ');
}

/** Convert Rust sequence type to component sequence type (add "Sequence" suffix) */
function normalizeSeqType(type: string): string {
    if (type.endsWith('Sequence')) return type;
    return type + 'Sequence';
}

// ─── map buildReport → IMFPackageViewer data ─────────────────────────────────

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mapReportToViewData(report: any): any {
    const pkg = report.package;
    const v = report.validation;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const flatIssues: any[] = [
        ...(v.critical ?? []),
        ...(v.errors ?? []),
        ...(v.warnings ?? []),
        ...(v.info ?? []),
    ];

    const hasCritical = (v.critical?.length ?? 0) > 0;
    const hasErrors = (v.errors?.length ?? 0) > 0;
    const valid = !hasCritical && !hasErrors;

    return {
        package: {
            assetMapId: pkg.assetMapId ?? '',
            volumeIndex: pkg.volumeIndex ?? 1,
            assetCount: pkg.assetCount ?? 0,
            cplCount: pkg.cplCount ?? 0,
            pklCount: pkg.pklCount ?? 0,
        },
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        cpls: (report.cpls ?? []).map((cpl: any) => {
            const editRate = normalizeEditRate(cpl.editRate);
            return {
                id: cpl.id ?? '',
                title: cpl.title ?? '',
                applicationProfile: cpl.applicationProfile ?? null,
                segmentCount: cpl.segmentCount ?? 0,
                timecodeStart: cpl.timecodeStart ?? null,
                isSupplemental: cpl.isSupplemental ?? false,
                markers: (cpl.markers ?? []).map((m: any) => ({ // eslint-disable-line @typescript-eslint/no-explicit-any
                    label: m.label ?? '',
                    offset: m.offset ?? null,
                    scope: m.annotation ?? null,
                })),
                sourceAsset: {
                    contentKind: null,
                    contentTitle: cpl.title ?? null,
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
                    audioLanguages: [],
                    subtitleLanguages: [],
                    forcedNarrativeLanguages: [],
                    audioType: null,
                    videoQuality: null,
                    videoDynamicRange: null,
                    tracks: { VIDEO: [], AUDIO: [], SUBTITLES: [], CAPTIONS: [], FORCED_NARRATIVE: [] },
                    sequences: (cpl.sequences ?? []).map((seq: any, i: number) => ({ // eslint-disable-line @typescript-eslint/no-explicit-any
                        type: normalizeSeqType(seq.type),
                        id: seq.id ?? '',
                        trackId: seq.trackId ?? '',
                        language: seq.language ?? null,
                        channelCount: seq.channelCount ?? null,
                        soundfield: seq.soundfield ?? null,
                        segmentId: null,
                        sequenceNumber: i,
                        sequenceResources: (seq.resources ?? []).map((r: any) => ({ // eslint-disable-line @typescript-eslint/no-explicit-any
                            id: r.id ?? '',
                            intrinsicDuration: r.intrinsicDuration ?? 0,
                            sourceDuration: r.sourceDuration ?? r.intrinsicDuration ?? 0,
                            sourceEncoding: r.sourceEncoding ?? null,
                            trackFileId: r.trackFileId ?? null,
                            editRate: normalizeEditRate(r.editRate) ?? editRate,
                            entryPoint: r.entryPoint ?? null,
                        })),
                    })),
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
            const report = mod.buildReport(xmlMapRef.current, { rules: rulesConfig });
            setPackageData(mapReportToViewData(report));
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
                const report = mod.buildReport(xmlMap, { rules: rulesConfigRef.current });
                setPackageData(mapReportToViewData(report));
            } catch (e) {
                console.error('[imf] buildReport error:', e);
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
