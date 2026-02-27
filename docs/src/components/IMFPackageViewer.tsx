import { useState } from 'react';

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
    markers: MarkerData[];
}

interface MarkerData {
    label: string;
    offset: number | null;
    scope: string | null;
}

// ─── icons ────────────────────────────────────────────────────────────────────

type IconProps = { width?: number; height?: number; className?: string };

const IPackage = (p: IconProps) => <svg {...{ width: 16, height: 16, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M16.5 9.4 7.55 4.24" /><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" /><polyline points="3.29 7 12 12 20.71 7" /><line x1="12" x2="12" y1="22" y2="12" /></svg>;
const ICheck = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><path d="m9 11 3 3L22 4" /></svg>;
const IAlert = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" /><path d="M12 9v4" /><path d="M12 17h.01" /></svg>;
const IChevron = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="m6 9 6 6 6-6" /></svg>;
const ILayers = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z" /><path d="m22.54 12.43-10 4.55a2 2 0 0 1-1.66 0l-9.4-4.28" /><path d="m22.54 16.43-10 4.55a2 2 0 0 1-1.66 0l-9.4-4.28" /></svg>;
const IMarker = (p: IconProps) => <svg {...{ width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, ...p }}><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" /><circle cx="12" cy="10" r="3" /></svg>;

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

// ─── CPL card ─────────────────────────────────────────────────────────────────

function CplCard({ cpl, isOpen, onToggle }: { cpl: CplEntry; isOpen: boolean; onToggle: () => void }) {
    const [activeTab, setActiveTab] = useState('metadata');

    const tabs = [
        { id: 'markers', label: `Markers${cpl.markers?.length ? ` (${cpl.markers.length})` : ''}`, icon: <IMarker /> },
        { id: 'metadata', label: 'Metadata', icon: <IPackage /> },
    ];

    return (
        <div className={`border rounded-xl overflow-hidden bg-white transition-all duration-200 ${isOpen ? 'border-zinc-300 shadow-sm ring-1 ring-zinc-200/50' : 'border-zinc-200 hover:border-zinc-300'}`}>
            <button onClick={onToggle} className="w-full flex items-center gap-3 px-4 py-3.5 text-left hover:bg-zinc-50 transition-colors">
                <span className={`flex transition-transform duration-200 ${isOpen ? '' : '-rotate-90'} ${isOpen ? 'text-amber-600' : 'text-zinc-400'}`}><IChevron /></span>
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                        {cpl.isSupplemental && <Badge variant="amber">Supplemental</Badge>}
                        {(cpl.unresolvedAncestorAssetIds?.length ?? 0) > 0 && <Badge variant="red">{cpl.unresolvedAncestorAssetIds.length} unresolved</Badge>}
                        {cpl.applicationProfile && <Badge variant="outline">{cpl.applicationProfile}</Badge>}
                        {cpl.segmentCount > 0 && <Badge variant="outline"><ILayers /> {cpl.segmentCount} segment{cpl.segmentCount !== 1 ? 's' : ''}</Badge>}
                    </div>
                    <div className="mt-1.5 text-xs font-mono text-zinc-400 truncate">{cpl.title}</div>
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
                                    { l: 'Type', v: cpl.isSupplemental ? 'Supplemental' : 'Original Version' },
                                    ...(cpl.issuer ? [{ l: 'Issuer', v: cpl.issuer }] : []),
                                    ...(cpl.creator ? [{ l: 'Creator', v: cpl.creator }] : []),
                                    ...(cpl.issueDate ? [{ l: 'Issue Date', v: cpl.issueDate }] : []),
                                    ...(cpl.applicationProfile ? [{ l: 'Application Profile', v: cpl.applicationProfile }] : []),
                                    ...(cpl.segmentCount != null ? [{ l: 'Segments', v: String(cpl.segmentCount) }] : []),
                                    ...(cpl.timecodeStart ? [{ l: 'Timecode Start', v: cpl.timecodeStart, m: true }] : []),
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
                        <CplCard key={cpl.id} cpl={cpl} isOpen={openCpls.has(cpl.id)} onToggle={() => toggleCpl(cpl.id)} />
                    ))}
                </div>
            </div>
        </div>
    );
}
