import { useState, useMemo } from 'react';

// ─── types ────────────────────────────────────────────────────────────────────

export interface ValidationIssue {
    severity: string;
    category: string;
    code: string;
    message: string;
    suggestion?: string | null;
    source?: string;
    cplId?: string;
}

export interface PackageViewData {
    package: {
        assetMapId: string;
        volumeIndex: number;
        assetCount: number;
        cplCount: number;
        pklCount: number;
    };
    cpls: CplEntry[];
    validation: {
        status: 'Valid' | 'ValidWithWarnings' | 'Invalid' | 'Error' | null;
        issues: ValidationIssue[];
        summary?: {
            total: number;
            critical: number;
            errors: number;
            warnings: number;
            info: number;
            is_playable: boolean;
            is_compliant: boolean;
        };
    };
}

interface CplEntry {
    id: string;
    title: string;
    issuer: string | null;
    creator: string | null;
    issueDate: string | null;
    applicationProfile: string | null;
    segmentCount: number;
    timecodeStart: string | null;
    isSupplemental: boolean;
    unresolvedAncestorAssetIds: string[];
    sourceAsset: SourceAssetData;
    markers: MarkerData[];
    deliveryComparison: Record<string, unknown> | null;
}

interface SourceAssetData {
    contentKind: string;
    contentTitle: string | null;
    territory: string | null;
    editRate: string | null;
    frameRate: string | null;
    duration: string | null;
    audioLanguages: string[];
    subtitleLanguages: string[];
    captionLanguages: string[];
    forcedNarrativeLanguages: string[];
    audioType: string | null;
    videoQuality: string | null;
    videoDynamicRange: string | null;
    tracks: {
        AUDIO: AudioTrackData[];
        VIDEO: VideoTrackData[];
        SUBTITLES: TimedTextTrackData[];
        CAPTIONS: TimedTextTrackData[];
        FORCED_NARRATIVE: TimedTextTrackData[];
    };
    sequences: SequenceData[];
}

interface SequenceData {
    type: string;
    id: string;
    trackId: string;
    segmentId: string;
    sequenceNumber: number;
    sequenceResources: SequenceResource[];
}

interface SequenceResource {
    id: string;
    intrinsicDuration: number;
    sourceDuration: number;
    sourceEncoding: string;
    trackFileId: string;
    annotation: string | null;
    editRate: string | null;
    entryPoint: number | null;
}

interface AudioTrackData {
    type: string;
    audioContentKind: string;
    channelCount: number;
    language: string;
    mcaTagName: string | null;
    mcaTagSymbol: string | null;
    atmosType: string | null;
    trackIdentifier: string;
    trackNumber: number;
    sequenceNumber: number | null;
    sequenceTrackId: string | null;
    fragmentDuration: string;
    trackDuration: number;
}

interface VideoTrackData {
    quality: string;
    dynamicRange: string;
    width: number;
    height: number;
    language?: string;
    trackIdentifier: string;
    trackNumber: number;
    sequenceNumber: number | null;
    sequenceTrackId: string | null;
    fragmentDuration: string;
    trackDuration: number;
}

interface TimedTextTrackData {
    language: string;
    trackIdentifier: string;
    trackNumber: number;
    sequenceNumber: number | null;
    sequenceTrackId: string | null;
    fragmentDuration: string;
    trackDuration: number;
}

interface MarkerData {
    label: string;
    offset: number | null;
    scope: string | null;
}

// ─── icons ────────────────────────────────────────────────────────────────────

type IconProps = { width?: number; height?: number; className?: string };

const IPackage = (p: IconProps) => <svg {...{ width: 16, height: 16, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M16.5 9.4 7.55 4.24" /><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" /><polyline points="3.29 7 12 12 20.71 7" /><line x1="12" x2="12" y1="22" y2="12" /></svg>;
const IFilm = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><rect width="18" height="18" x="3" y="3" rx="2" /><path d="M7 3v18" /><path d="M3 7.5h4" /><path d="M3 12h18" /><path d="M3 16.5h4" /><path d="M17 3v18" /><path d="M17 7.5h4" /><path d="M17 16.5h4" /></svg>;
const IMusic = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>;
const ICheck = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><path d="m9 11 3 3L22 4" /></svg>;
const IAlert = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" /><path d="M12 9v4" /><path d="M12 17h.01" /></svg>;
const IChevron = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="m6 9 6 6 6-6" /></svg>;
const ILayers = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z" /><path d="m22.54 12.43-10 4.55a2 2 0 0 1-1.66 0l-9.4-4.28" /><path d="m22.54 16.43-10 4.55a2 2 0 0 1-1.66 0l-9.4-4.28" /></svg>;
const IGlobe = (p: IconProps) => <svg {...{ width: 12, height: 12, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><circle cx="12" cy="12" r="10" /><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" /><path d="M2 12h20" /></svg>;
const IClock = (p: IconProps) => <svg {...{ width: 12, height: 12, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>;
const IMonitor = (p: IconProps) => <svg {...{ width: 12, height: 12, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><rect width="20" height="14" x="2" y="3" rx="2" /><line x1="8" x2="16" y1="21" y2="21" /><line x1="12" x2="12" y1="17" y2="21" /></svg>;
const IMarker = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" /><circle cx="12" cy="10" r="3" /></svg>;

// ─── helpers ──────────────────────────────────────────────────────────────────

const seqTypeLabel = (t: string) => ({ MainImageSequence: 'Video', MainAudioSequence: 'Audio', SubtitlesSequence: 'Subtitles', HearingImpairedCaptionsSequence: 'HI Captions', ForcedNarrativeSequence: 'Forced Narrative', IABSequence: 'IAB Audio' } as Record<string, string>)[t] ?? t;

const seqTypeColor = (type: string) => ({
    MainImageSequence: { fill: '#3b82f6', fillBg: 'rgba(59,130,246,0.12)' },
    MainAudioSequence: { fill: '#a855f7', fillBg: 'rgba(168,85,247,0.1)' },
    SubtitlesSequence: { fill: '#22c55e', fillBg: 'rgba(34,197,94,0.1)' },
    ForcedNarrativeSequence: { fill: '#f97316', fillBg: 'rgba(249,115,22,0.1)' },
    HearingImpairedCaptionsSequence: { fill: '#eab308', fillBg: 'rgba(234,179,8,0.1)' },
    IABSequence: { fill: '#ec4899', fillBg: 'rgba(236,72,153,0.1)' },
} as Record<string, { fill: string; fillBg: string }>)[type] ?? { fill: '#94a3b8', fillBg: 'rgba(148,163,184,0.1)' };

const audioTypeLabel = (t: string | null) => ({ DOLBY_ATMOS: 'Dolby Atmos', DOLBY_DIGITAL_PLUS: 'DD+', DOLBY_DIGITAL: 'DD', STEREO: 'Stereo' } as Record<string, string>)[t ?? ''] ?? (t ?? '—');
const dynRangeLabel = (t: string | null) => ({ SDR: 'SDR', HDR10: 'HDR10', HDR_DOLBY_VISION: 'Dolby Vision', HLG: 'HLG' } as Record<string, string>)[t ?? ''] ?? (t ?? '—');
const contentKindLabel = (t: string | null) => ({ PRM: 'Primary', VI: 'Visually Impaired', HI: 'Hearing Impaired', CM: 'Commentary' } as Record<string, string>)[t ?? ''] ?? (t ?? '—');
const truncUuid = (u: string | null | undefined) => u ? u.substring(0, 8) + '…' : '—';

const toSeconds = (count: number | null | undefined, er: string | null | undefined): number => {
    if (count == null || !er) return 0;
    const p = er.trim().split(/\s+/), n = +p[0], d = p[1] ? +p[1] : 1;
    return (!n || !d) ? 0 : count / (n / d);
};

const durationToTC = (count: number | null | undefined, er: string | null | undefined): string => {
    if (count == null || !er) return '—';
    const p = er.trim().split(/\s+/), n = +p[0], d = p[1] ? +p[1] : 1;
    if (!n || !d) return String(count);
    const ts = count / (n / d);
    if (n / d >= 8000) {
        // audio: hh:mm:ss.mmm
        return `${String(Math.floor(ts / 3600)).padStart(2, '0')}:${String(Math.floor((ts % 3600) / 60)).padStart(2, '0')}:${String(Math.floor(ts % 60)).padStart(2, '0')}.${String(Math.round((ts - Math.floor(ts)) * 1000)).padStart(3, '0')}`;
    }
    // video: hh:mm:ss:ff
    const fps = n / d;
    return `${String(Math.floor(ts / 3600)).padStart(2, '0')}:${String(Math.floor((ts % 3600) / 60)).padStart(2, '0')}:${String(Math.floor(ts % 60)).padStart(2, '0')}:${String(Math.round((ts - Math.floor(ts)) * fps)).padStart(2, '0')}`;
};

// ─── badge ────────────────────────────────────────────────────────────────────

const badgeVariants: Record<string, string> = {
    default: 'bg-zinc-100 text-zinc-500',
    blue: 'bg-blue-500/10 text-blue-500 border border-blue-500/20',
    purple: 'bg-purple-500/10 text-purple-500 border border-purple-500/20',
    green: 'bg-green-500/10 text-green-500 border border-green-500/20',
    amber: 'bg-yellow-500/10 text-yellow-600 border border-yellow-500/20',
    red: 'bg-red-500/10 text-red-500 border border-red-500/20',
    pink: 'bg-pink-500/10 text-pink-500 border border-pink-500/20',
    outline: 'bg-transparent text-zinc-600 border border-zinc-200',
};

function Badge({ children, variant = 'default' }: { children: React.ReactNode; variant?: string }) {
    return (
        <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-medium leading-[18px] whitespace-nowrap ${badgeVariants[variant] ?? badgeVariants.default}`}>
            {children}
        </span>
    );
}

function Th({ children }: { children: React.ReactNode }) {
    return <th className="px-3 py-2 text-left text-[10px] font-semibold text-zinc-400 uppercase tracking-wider border-b border-zinc-200">{children}</th>;
}

// ─── sequence timeline ────────────────────────────────────────────────────────

function SequenceTimeline({ sequences, maxDuration, tracks, editRate, issues }: {
    sequences: SequenceData[];
    maxDuration: number;
    tracks: SourceAssetData['tracks'];
    editRate: string | null;
    issues?: ValidationIssue[];
}) {
    const [expanded, setExpanded] = useState<Set<string | number>>(new Set());

    if (!sequences?.length) return null;

    const seqMeta = useMemo(() => {
        const lang: Record<number, string> = {}, mca: Record<number, string> = {}, kind: Record<number, string> = {};
        for (const t of [...(tracks.AUDIO ?? []), ...(tracks.SUBTITLES ?? []), ...(tracks.CAPTIONS ?? []), ...(tracks.FORCED_NARRATIVE ?? [])]) {
            if (t.sequenceNumber != null && t.language && !lang[t.sequenceNumber]) lang[t.sequenceNumber] = t.language;
        }
        for (const t of (tracks.AUDIO ?? [])) {
            if (t.sequenceNumber != null) {
                if (t.mcaTagName && !mca[t.sequenceNumber]) mca[t.sequenceNumber] = t.mcaTagName;
                if (t.audioContentKind && !kind[t.sequenceNumber]) kind[t.sequenceNumber] = t.audioContentKind;
            }
        }
        return { lang, mca, kind };
    }, [tracks]);

    const buildLabel = (seq: SequenceData) => {
        const parts = [seqTypeLabel(seq.type)];
        const l = seqMeta.lang[seq.sequenceNumber], m = seqMeta.mca[seq.sequenceNumber], k = seqMeta.kind[seq.sequenceNumber];
        if (l) parts.push(l.toUpperCase());
        if (m) parts.push(m);
        if (k && k !== 'PRM') parts.push(contentKindLabel(k));
        return parts.join(' · ');
    };

    const toggle = (id: string | number) => setExpanded(p => { const n = new Set(p); n.has(id) ? n.delete(id) : n.add(id); return n; });

    const issueColors = (severity: string) => {
        if (severity === 'Warning') return { bg: 'bg-amber-50', border: 'rgba(245,158,11,0.35)', fill: 'rgba(245,158,11,0.1)', line: '#f59e0b', lineFaded: '#f59e0b40', text: 'text-amber-500' };
        if (severity === 'Info')    return { bg: 'bg-blue-50',  border: 'rgba(59,130,246,0.35)',  fill: 'rgba(59,130,246,0.1)',  line: '#3b82f6', lineFaded: '#3b82f640', text: 'text-blue-500' };
        /* Error / Critical */      return { bg: 'bg-red-50',   border: 'rgba(239,68,68,0.35)',   fill: 'rgba(239,68,68,0.1)',   line: '#ef4444', lineFaded: '#ef444440', text: 'text-red-500' };
    };

    const resourceRows = (seq: SequenceData) => {
        let o = 0;
        return seq.sequenceResources.map(r => { const row = { ...r, _offset: o }; o += r.sourceDuration ?? 0; return row; });
    };

    return (
        <div className="flex flex-col gap-0.5">
            {sequences.map((seq, i) => {
                const c = seqTypeColor(seq.type);
                const totalSec = seq.sequenceResources.reduce((s, r) => s + toSeconds(r.sourceDuration, r.editRate ?? editRate), 0);
                const fillPct = maxDuration > 0 ? Math.max((totalSec / maxDuration) * 100, 2) : 100;
                const isOpen = expanded.has(seq.id || i);
                const seqId = seq.id || i;
                const shortId = seq.trackId?.substring(0, 8) ?? '';
                const segDurIssue = issues?.find(iss => iss.code.endsWith('/SegmentDuration') && shortId && iss.message.includes(shortId));
                const ic = segDurIssue ? issueColors(segDurIssue.severity) : null;
                const durLabel = totalSec > 0 ? `${totalSec.toFixed(3)}s` : '—';
                return (
                    <div key={seqId}>
                        <div onClick={() => toggle(seqId)} className="flex items-center gap-2 cursor-pointer rounded py-0.5 hover:bg-zinc-50 transition-colors">
                            <div className="w-48 flex-shrink-0 flex items-center gap-1.5 pl-1">
                                <span className={`flex transition-transform duration-200 ${isOpen ? '' : '-rotate-90'}`} style={{ color: c.fill }}><IChevron /></span>
                                <span className="text-[11px] font-semibold truncate" style={{ color: c.fill }}>{buildLabel(seq)}</span>
                            </div>
                            <div className={`flex-1 h-5 rounded overflow-hidden relative ${ic ? ic.bg : 'bg-zinc-100'}`}
                                style={{ border: ic ? `1px solid ${ic.border}` : '1px solid rgba(228,228,231,0.5)' }}>
                                <div className="h-full rounded relative flex" style={{ width: `${fillPct}%` }}>
                                    {seq.sequenceResources.map((res, ri) => {
                                        const td = seq.sequenceResources.reduce((s, r) => s + toSeconds(r.sourceDuration, r.editRate ?? editRate), 0);
                                        const rs = toSeconds(res.sourceDuration, res.editRate ?? editRate);
                                        const pct = td > 0 ? (rs / td) * 100 : 100;
                                        return (
                                            <div key={ri} className="h-full relative flex items-center"
                                                style={{ width: `${Math.max(pct, 6)}%`, minWidth: '20px', background: ic ? ic.fill : c.fillBg, borderLeft: ri === 0 ? `2.5px solid ${ic ? ic.line : c.fill}` : `1px solid ${ic ? ic.lineFaded : c.fill + '40'}` }}>
                                                <span className="text-[9px] font-semibold px-1.5 truncate" style={{ color: ic ? ic.line : c.fill }}>
                                                    {seq.sequenceResources.length > 1 ? `R${ri + 1}` : ''}
                                                </span>
                                            </div>
                                        );
                                    })}
                                </div>
                            </div>
                            <div className={`w-24 flex-shrink-0 text-right pr-2 text-[10px] font-mono ${ic ? ic.text + ' font-semibold' : 'text-zinc-400'}`}
                                title={segDurIssue ? segDurIssue.message : undefined}>
                                {segDurIssue && <span className="mr-1">⚠</span>}{durLabel}
                            </div>
                        </div>
                        {isOpen && (
                            <div className="ml-[208px] mr-11 mt-0.5 mb-1.5 border border-zinc-200 rounded-md overflow-hidden text-[11px]">
                                <table className="w-full border-collapse">
                                    <thead><tr className="bg-zinc-50">
                                        {['#', 'Offset', 'Track File', 'Duration', 'Entry Point', 'Encoding'].map(h => (
                                            <th key={h} className="px-2.5 py-1 text-left text-[10px] font-semibold text-zinc-400 uppercase tracking-wider border-b border-zinc-200">{h}</th>
                                        ))}
                                    </tr></thead>
                                    <tbody>
                                        {resourceRows(seq).map((res, ri) => {
                                            const rr = res.editRate ?? editRate;
                                            return (
                                                <tr key={res.id || ri} className="border-b border-zinc-100 last:border-0 hover:bg-zinc-50/50">
                                                    <td className="px-2.5 py-1 font-mono text-zinc-400">
                                                        <span className="px-1.5 py-px rounded text-[10px] font-semibold" style={{ background: c.fillBg, color: c.fill }}>R{ri + 1}</span>
                                                    </td>
                                                    <td className="px-2.5 py-1 font-mono">{durationToTC(res._offset, rr)}</td>
                                                    <td className="px-2.5 py-1 font-mono text-zinc-400">{truncUuid(res.trackFileId)}</td>
                                                    <td className="px-2.5 py-1 font-mono">
                                                        {durationToTC(res.sourceDuration, rr)}
                                                        {res.intrinsicDuration != null && res.sourceDuration !== res.intrinsicDuration && (
                                                            <span className="text-zinc-400 text-[10px]"> / {durationToTC(res.intrinsicDuration, rr)}</span>
                                                        )}
                                                    </td>
                                                    <td className="px-2.5 py-1 font-mono text-zinc-400">{res.entryPoint != null ? durationToTC(res.entryPoint, rr) : '—'}</td>
                                                    <td className="px-2.5 py-1 font-mono text-[10px] text-zinc-400">{res.sourceEncoding ? truncUuid(res.sourceEncoding) : '—'}</td>
                                                </tr>
                                            );
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        )}
                    </div>
                );
            })}
        </div>
    );
}

// ─── track table ──────────────────────────────────────────────────────────────

function TrackTable({ tracks }: { tracks: SourceAssetData['tracks'] }) {
    const all = useMemo(() => {
        const r: Array<(AudioTrackData | VideoTrackData | TimedTextTrackData) & { _cat: string }> = [];
        (['VIDEO', 'AUDIO', 'SUBTITLES', 'CAPTIONS', 'FORCED_NARRATIVE'] as const).forEach(cat =>
            (tracks[cat] ?? []).forEach((t: AudioTrackData | VideoTrackData | TimedTextTrackData) => r.push({ ...t, _cat: cat }))
        );
        return r;
    }, [tracks]);

    if (!all.length) return null;

    const cc: Record<string, string> = { VIDEO: 'blue', AUDIO: 'purple', SUBTITLES: 'green', CAPTIONS: 'amber', FORCED_NARRATIVE: 'amber' };

    return (
        <div className="border border-zinc-200 rounded-lg overflow-hidden text-xs">
            <table className="w-full border-collapse">
                <thead>
                    <tr className="bg-zinc-50">
                        {['#', 'Type', 'Codec / Format', 'Details', 'Language', 'Fragment', 'Track ID', 'Seq Track'].map(h => <Th key={h}>{h}</Th>)}
                    </tr>
                </thead>
                <tbody>
                    {all.map((t, i) => {
                        const cat = t._cat;
                        const video = cat === 'VIDEO' ? t as VideoTrackData & { _cat: string } : null;
                        const audio = cat === 'AUDIO' ? t as AudioTrackData & { _cat: string } : null;
                        return (
                            <tr key={i} className="border-b border-zinc-100 last:border-0 hover:bg-zinc-50 transition-colors">
                                <td className="px-3 py-1.5 font-mono text-zinc-400">{t.trackNumber}</td>
                                <td className="px-3 py-1.5"><Badge variant={cc[cat] ?? 'default'}>{cat === 'FORCED_NARRATIVE' ? 'FN' : cat}</Badge></td>
                                <td className="px-3 py-1.5">
                                    {video ? (
                                        <span className="flex items-center gap-1.5">
                                            <span className="font-medium">{video.width}×{video.height}</span>
                                            <Badge variant="outline">{video.quality}</Badge>
                                            <Badge variant={(video.dynamicRange?.includes('HDR') || video.dynamicRange?.includes('DOLBY')) ? 'pink' : 'outline'}>{dynRangeLabel(video.dynamicRange)}</Badge>
                                        </span>
                                    ) : audio ? (
                                        <span className="flex items-center gap-1.5">
                                            <span className="font-medium">{audioTypeLabel(audio.type)}</span>
                                            {audio.mcaTagName ? <Badge variant="outline">{audio.mcaTagName}</Badge>
                                                : audio.channelCount > 0 ? <Badge variant="outline">{audio.channelCount === 6 ? '5.1' : audio.channelCount === 8 ? '7.1' : `${audio.channelCount}ch`}</Badge>
                                                    : null}
                                            {audio.atmosType && <Badge variant="purple">{audio.atmosType}</Badge>}
                                        </span>
                                    ) : (
                                        <span className="font-medium">TTML</span>
                                    )}
                                </td>
                                <td className="px-3 py-1.5 text-zinc-500">{audio ? contentKindLabel(audio.audioContentKind) : '—'}</td>
                                <td className="px-3 py-1.5">
                                    {t.language ? <span className="inline-flex items-center gap-1"><IGlobe />{t.language.toUpperCase()}</span> : '—'}
                                </td>
                                <td className="px-3 py-1.5 font-mono text-[11px] text-zinc-500">{t.fragmentDuration || '—'}</td>
                                <td className="px-3 py-1.5 font-mono text-[11px] text-zinc-400">{truncUuid(t.trackIdentifier)}</td>
                                <td className="px-3 py-1.5 font-mono text-[11px] text-zinc-400">{t.sequenceTrackId ? truncUuid(t.sequenceTrackId) : '—'}</td>
                            </tr>
                        );
                    })}
                </tbody>
            </table>
        </div>
    );
}

// ─── CPL card ─────────────────────────────────────────────────────────────────

function CplCard({ cpl, isOpen, onToggle, issues }: { cpl: CplEntry; isOpen: boolean; onToggle: () => void; issues?: ValidationIssue[] }) {
    const sa = cpl.sourceAsset;
    const [activeTab, setActiveTab] = useState('timeline');

    const maxDuration = useMemo(() => {
        if (!sa?.sequences) return 0;
        return Math.max(...sa.sequences.map(s => s.sequenceResources.reduce((sum, r) => sum + toSeconds(r.sourceDuration, r.editRate ?? sa.editRate), 0)));
    }, [sa?.sequences, sa?.editRate]);

    const kc: Record<string, { badge: string; icon: string }> = {
        FEATURE: { badge: 'blue', icon: '🎬' },
        TRAILER: { badge: 'purple', icon: '🎞️' },
        SHORT: { badge: 'green', icon: '📽️' },
    };
    const k = kc[sa?.contentKind ?? ''] ?? { badge: 'default', icon: '📦' };

    const tabs = [
        { id: 'timeline', label: 'Timeline', icon: <ILayers /> },
        { id: 'tracks', label: 'Tracks', icon: <IMusic /> },
        { id: 'markers', label: `Markers${cpl.markers?.length ? ` (${cpl.markers.length})` : ''}`, icon: <IMarker /> },
        { id: 'metadata', label: 'Metadata', icon: <IPackage /> },
    ];

    return (
        <div className={`border rounded-xl overflow-hidden bg-white transition-all duration-200 ${isOpen ? 'border-zinc-300 shadow-sm ring-1 ring-zinc-200/50' : 'border-zinc-200 hover:border-zinc-300'}`}>
            <button onClick={onToggle} className="w-full flex items-center gap-3 px-4 py-3.5 text-left hover:bg-zinc-50 transition-colors">
                <span className={`flex transition-transform duration-200 ${isOpen ? '' : '-rotate-90'} ${isOpen ? 'text-amber-600' : 'text-zinc-400'}`}><IChevron /></span>
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                        <Badge variant={k.badge}>{k.icon} {sa?.contentKind ?? '—'}</Badge>
                        {cpl.isSupplemental && <Badge variant="amber">Supplemental</Badge>}
                        {(cpl.unresolvedAncestorAssetIds?.length ?? 0) > 0 && <Badge variant="red">⚠ {cpl.unresolvedAncestorAssetIds.length} unresolved</Badge>}
                        {sa?.territory && <Badge variant="outline"><IGlobe /> {sa.territory}</Badge>}
                        {sa?.duration && <Badge variant="outline"><IClock /> {sa.duration}</Badge>}
                        {sa?.videoQuality && <Badge variant="outline"><IMonitor /> {sa.videoQuality} {dynRangeLabel(sa.videoDynamicRange)}</Badge>}
                        {sa?.frameRate && <Badge variant="outline">{sa.frameRate} fps</Badge>}
                        {sa?.audioType && sa.audioType !== 'STEREO' && <Badge variant="purple">{audioTypeLabel(sa.audioType)}</Badge>}
                        {cpl.applicationProfile && <Badge variant="outline">{cpl.applicationProfile}</Badge>}
                    </div>
                    <div className="mt-1.5 text-xs font-mono text-zinc-400 truncate">{cpl.title}</div>
                </div>
                <div className="flex gap-1 flex-shrink-0 items-center text-[11px] text-zinc-400">
                    <span className="flex items-center gap-0.5"><IFilm /> {sa?.tracks?.VIDEO?.length ?? 0}</span>
                    <span className="px-0.5 opacity-30">·</span>
                    <span className="flex items-center gap-0.5"><IMusic /> {sa?.tracks?.AUDIO?.length ?? 0}</span>
                    <span className="px-0.5 opacity-30">·</span>
                    <span className="flex items-center gap-0.5"><ILayers /> {sa?.sequences?.length ?? 0}</span>
                </div>
            </button>
            {isOpen && (
                <div className="border-t border-zinc-200">
                    <div className="flex border-b border-zinc-200 bg-zinc-50 px-3">
                        {tabs.map(tab => (
                            <button key={tab.id} onClick={() => setActiveTab(tab.id)}
                                className={`flex items-center gap-1.5 px-3.5 py-2.5 text-xs font-medium transition-colors -mb-px ${activeTab === tab.id ? 'text-amber-600 border-b-2 border-amber-600' : 'text-zinc-400 border-b-2 border-transparent hover:text-zinc-600'}`}>
                                {tab.icon} {tab.label}
                            </button>
                        ))}
                    </div>
                    <div className="p-4">
                        {activeTab === 'timeline' && sa?.sequences && (
                            <SequenceTimeline sequences={sa.sequences} maxDuration={maxDuration} tracks={sa.tracks} editRate={sa.editRate} issues={issues} />
                        )}
                        {activeTab === 'tracks' && sa?.tracks && <TrackTable tracks={sa.tracks} />}
                        {activeTab === 'markers' && (
                            cpl.markers?.length > 0 ? (
                                <div className="border border-zinc-200 rounded-lg overflow-hidden text-xs">
                                    <table className="w-full border-collapse">
                                        <thead><tr className="bg-zinc-50">{['Label', 'Offset', 'Scope'].map(h => <Th key={h}>{h}</Th>)}</tr></thead>
                                        <tbody>
                                            {cpl.markers.map((m, i) => (
                                                <tr key={i} className="border-b border-zinc-100 last:border-0 hover:bg-zinc-50">
                                                    <td className="px-3 py-1.5 font-medium">{m.label || '—'}</td>
                                                    <td className="px-3 py-1.5 font-mono text-zinc-500">{m.offset ?? '—'}</td>
                                                    <td className="px-3 py-1.5 text-zinc-500">{m.scope || '—'}</td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            ) : <p className="text-xs text-zinc-400 text-center py-8">No markers defined in this CPL.</p>
                        )}
                        {activeTab === 'metadata' && (
                            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                                {[
                                    { l: 'CPL UUID', v: cpl.id.replace('urn:uuid:', ''), m: true },
                                    ...(sa?.contentTitle ? [{ l: 'Content Title', v: sa.contentTitle }] : []),
                                    { l: 'Edit Rate', v: sa?.editRate ?? '—' },
                                    { l: 'Frame Rate', v: sa?.frameRate ? `${sa.frameRate} fps` : '—' },
                                    { l: 'Type', v: cpl.isSupplemental ? 'Supplemental' : 'Original Version' },
                                    ...(cpl.issuer ? [{ l: 'Issuer', v: cpl.issuer }] : []),
                                    ...(cpl.creator ? [{ l: 'Creator', v: cpl.creator }] : []),
                                    ...(cpl.issueDate ? [{ l: 'Issue Date', v: cpl.issueDate }] : []),
                                    ...(cpl.applicationProfile ? [{ l: 'Application Profile', v: cpl.applicationProfile }] : []),
                                    ...(cpl.segmentCount != null ? [{ l: 'Segments', v: String(cpl.segmentCount) }] : []),
                                    ...(cpl.timecodeStart ? [{ l: 'Timecode Start', v: cpl.timecodeStart, m: true }] : []),
                                    { l: 'Audio Languages', v: sa?.audioLanguages?.map(l => l.toUpperCase()).join(', ') || '—' },
                                    { l: 'Subtitle Languages', v: sa?.subtitleLanguages?.map(l => l.toUpperCase()).join(', ') || '—' },
                                    { l: 'Forced Narrative', v: sa?.forcedNarrativeLanguages?.map(l => l.toUpperCase()).join(', ') || '—' },
                                ].map(item => (
                                    <div key={item.l} className="px-3 py-2.5 rounded-lg bg-zinc-50 border border-zinc-100">
                                        <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 mb-1">{item.l}</div>
                                        <div className={`text-xs font-medium text-zinc-900 break-all ${item.m ? 'font-mono' : ''}`}>{item.v}</div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

// ─── main component ───────────────────────────────────────────────────────────

export default function IMFPackageViewer({ data }: { data: PackageViewData }) {
    const [openCpls, setOpenCpls] = useState<Set<string>>(() => new Set([data?.cpls?.[0]?.id].filter(Boolean) as string[]));
    const [showValidation, setShowValidation] = useState(false);

    const toggleCpl = (id: string) => setOpenCpls(p => { const n = new Set(p); n.has(id) ? n.delete(id) : n.add(id); return n; });
    const expandAll = () => setOpenCpls(new Set(data?.cpls?.map(c => c.id) ?? []));
    const collapseAll = () => setOpenCpls(new Set());

    const v = data.validation;
    const vStatus = v?.status;
    const vIsValid = vStatus === 'Valid';
    const vIsWarning = vStatus === 'ValidWithWarnings';
    const vIsInvalid = vStatus === 'Invalid' || vStatus === 'Error';
    const sum = v?.summary;
    const criticalCount = sum?.critical ?? v?.issues.filter(i => i.severity === 'Critical').length ?? 0;
    const errorCount    = sum?.errors   ?? v?.issues.filter(i => i.severity === 'Error').length   ?? 0;
    const warningCount  = sum?.warnings ?? v?.issues.filter(i => i.severity === 'Warning').length ?? 0;
    const hardCount = criticalCount + errorCount;
    const vLabel = vIsValid ? 'Valid'
        : vIsWarning  ? `${warningCount} Warning${warningCount !== 1 ? 's' : ''}`
        : vIsInvalid  ? `${hardCount} Error${hardCount !== 1 ? 's' : ''}`
        : null;

    const severityColor = (s: string) => ({ Critical: 'text-red-700', Error: 'text-red-600', Warning: 'text-amber-600', Info: 'text-blue-500' } as Record<string, string>)[s] ?? 'text-zinc-500';
    const severityBg = (s: string) => ({ Critical: 'bg-red-100 text-red-700', Error: 'bg-red-50 text-red-600', Warning: 'bg-amber-50 text-amber-600', Info: 'bg-blue-50 text-blue-500' } as Record<string, string>)[s] ?? 'bg-zinc-50 text-zinc-500';

    return (
        <div className="bg-white rounded-xl overflow-hidden border border-zinc-200 shadow-sm">
            <div className="max-w-full px-4 py-5">
                {/* Header */}
                <div className="flex items-start justify-between mb-5 gap-4 flex-wrap">
                    <div className="flex items-center gap-2.5">
                        <div className="w-8 h-8 rounded-lg bg-amber-600 text-white flex items-center justify-center">
                            <IPackage />
                        </div>
                        <div>
                            <h2 className="text-base font-bold leading-tight text-zinc-900">IMF Package</h2>
                            <p className="text-xs text-zinc-400 font-mono">{data.package.assetMapId.replace('urn:uuid:', '')}</p>
                        </div>
                    </div>
                    {vStatus && vLabel && (
                        <button
                            onClick={() => setShowValidation(s => !s)}
                            className={`flex items-center gap-2 px-3.5 py-2 rounded-lg border font-semibold text-[13px] leading-none transition-all cursor-pointer ${vIsValid ? 'bg-green-500/5 text-green-600 border-green-500/25 hover:bg-green-500/10' : vIsWarning ? 'bg-amber-500/5 text-amber-600 border-amber-500/25 hover:bg-amber-500/10' : 'bg-red-500/5 text-red-600 border-red-500/25 hover:bg-red-500/10'}`}>
                            {vIsValid ? <ICheck /> : <IAlert />}
                            {vLabel}
                            <span className={`flex opacity-60 transition-transform duration-200 ${showValidation ? '' : '-rotate-90'}`}><IChevron /></span>
                        </button>
                    )}
                </div>

                {/* Validation panel */}
                {showValidation && vStatus && (
                    <div className={`mb-4 px-4 py-3.5 rounded-lg border ${vIsValid ? 'bg-green-50/50 border-green-500/15' : vIsWarning ? 'bg-amber-50/50 border-amber-500/15' : 'bg-red-50/50 border-red-500/15'}`}>
                        <div className={`text-[11px] font-semibold uppercase tracking-wider mb-2 ${vIsValid ? 'text-green-600' : vIsWarning ? 'text-amber-600' : 'text-red-600'}`}>
                            ST 2067 validation · {vStatus}
                        </div>
                        {/* Summary row */}
                        {sum && (
                            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 mb-2.5 pb-2.5 border-b border-zinc-100">
                                {sum.critical  > 0 && <span className="text-[11px] font-semibold text-red-700">{sum.critical} Critical</span>}
                                {sum.errors    > 0 && <span className="text-[11px] font-semibold text-red-600">{sum.errors} Error{sum.errors !== 1 ? 's' : ''}</span>}
                                {sum.warnings  > 0 && <span className="text-[11px] font-semibold text-amber-600">{sum.warnings} Warning{sum.warnings !== 1 ? 's' : ''}</span>}
                                {sum.info      > 0 && <span className="text-[11px] font-semibold text-blue-500">{sum.info} Info</span>}
                                <span className="ml-auto flex items-center gap-2">
                                    <span className={`text-[11px] font-medium ${sum.is_playable ? 'text-green-600' : 'text-red-600'}`}>
                                        {sum.is_playable ? '▶ Playable' : '✕ Not playable'}
                                    </span>
                                    <span className={`text-[11px] font-medium ${sum.is_compliant ? 'text-green-600' : 'text-amber-600'}`}>
                                        {sum.is_compliant ? '✓ Compliant' : '! Non-compliant'}
                                    </span>
                                </span>
                            </div>
                        )}
                        {v.issues.length === 0
                            ? <p className="text-xs text-zinc-500 leading-relaxed">No issues found across {data.package.cplCount} composition playlist{data.package.cplCount !== 1 ? 's' : ''}.</p>
                            : (
                                <div className="flex flex-col gap-1.5">
                                    {v.issues.map((issue, i) => (
                                        <div key={i} className="flex items-start gap-2 text-xs leading-relaxed">
                                            <span className={`flex-shrink-0 px-1.5 py-0.5 rounded text-[10px] font-semibold mt-0.5 ${severityBg(issue.severity)}`}>{issue.severity}</span>
                                            <div className="flex flex-col gap-0.5 min-w-0">
                                                <span className={`font-mono text-[10px] ${severityColor(issue.severity)}`}>{issue.code}</span>
                                                <span className="text-zinc-600">{issue.message}</span>
                                                {issue.suggestion && <span className="text-zinc-400 text-[11px]">{issue.suggestion}</span>}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            )}
                    </div>
                )}

                {/* Stats */}
                <div className="grid grid-cols-3 gap-2 mb-5">
                    {[
                        { l: 'CPLs', v: data.package.cplCount },
                        { l: 'Assets', v: data.package.assetCount },
                        { l: 'PKLs', v: data.package.pklCount },
                    ].map(s => (
                        <div key={s.l} className="px-3 py-2.5 rounded-lg border border-zinc-200 bg-white">
                            <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 mb-0.5">{s.l}</div>
                            <div className="text-sm font-semibold text-zinc-900 truncate">{String(s.v)}</div>
                        </div>
                    ))}
                </div>

                {/* CPL list */}
                <div className="flex items-center justify-between mb-2.5">
                    <h3 className="text-sm font-semibold flex items-center gap-1.5 text-zinc-700">
                        <span className="text-amber-600"><ILayers /></span>
                        Composition Playlists
                        <span className="font-normal text-zinc-400">({data.cpls.length})</span>
                    </h3>
                    <div className="flex gap-1">
                        <button onClick={expandAll} className="px-2.5 py-1 text-[11px] font-medium border border-zinc-200 rounded-md bg-white text-zinc-400 hover:bg-zinc-50 hover:text-amber-600 cursor-pointer transition-colors">Expand All</button>
                        <button onClick={collapseAll} className="px-2.5 py-1 text-[11px] font-medium border border-zinc-200 rounded-md bg-white text-zinc-400 hover:bg-zinc-50 hover:text-amber-600 cursor-pointer transition-colors">Collapse</button>
                    </div>
                </div>

                <div className="flex flex-col gap-3">
                    {data.cpls.map(cpl => (
                        <CplCard key={cpl.id} cpl={cpl} isOpen={openCpls.has(cpl.id)} onToggle={() => toggleCpl(cpl.id)}
                            issues={v?.issues?.filter(i => !i.cplId || i.cplId === cpl.id || i.cplId.includes(cpl.id.replace('urn:uuid:', '').substring(0, 8)))} />
                    ))}
                </div>
            </div>
        </div>
    );
}
