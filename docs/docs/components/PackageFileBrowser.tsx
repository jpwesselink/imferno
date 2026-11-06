import { useEffect, useMemo, useState } from 'react';
import { highlightXml } from './highlight';

// Rendered as a section inside IMFPackageViewer, so it uses the viewer's
// Tailwind vocabulary (zinc palette, `dark:` = html.rp-dark, rounded-xl
// cards, amber section icons, the same badge colours).

export interface BrowserFile {
    name: string;
    text: string;
}

type Kind = 'VOLINDEX' | 'ASSETMAP' | 'PKL' | 'CPL' | 'OPL' | 'XML';

function kindOf(name: string): Kind {
    const u = name.toUpperCase();
    if (u === 'VOLINDEX.XML') return 'VOLINDEX';
    if (u === 'ASSETMAP.XML') return 'ASSETMAP';
    if (u.startsWith('PKL_')) return 'PKL';
    if (u.startsWith('OPL_')) return 'OPL';
    if (u.startsWith('CPL_')) return 'CPL';
    return 'XML';
}

// Package order: how a receiver walks an IMP (volume → asset map →
// packing list → compositions → output profiles).
const ORDER: Kind[] = ['VOLINDEX', 'ASSETMAP', 'PKL', 'CPL', 'OPL', 'XML'];

// Same colour vocabulary as the viewer's Badge variants.
const KIND_CLASS: Record<Kind, string> = {
    VOLINDEX: 'bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400',
    ASSETMAP: 'bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400',
    PKL: 'bg-blue-500/10 text-blue-500 border border-blue-500/20',
    CPL: 'bg-yellow-500/10 text-yellow-600 border border-yellow-500/20',
    OPL: 'bg-purple-500/10 text-purple-500 border border-purple-500/20',
    XML: 'bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400',
};

const FileCodeIcon = () => (
    <svg width={14} height={14} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
        <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
        <path d="M14 2v4a2 2 0 0 0 2 2h4" />
        <path d="m10 13-2 2 2 2" />
        <path d="m14 17 2-2-2-2" />
    </svg>
);

function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    return `${(bytes / 1024).toFixed(1)} KB`;
}

export default function PackageFileBrowser({ files }: { files: BrowserFile[] }) {
    const sorted = useMemo(
        () =>
            [...files].sort(
                (a, b) =>
                    ORDER.indexOf(kindOf(a.name)) - ORDER.indexOf(kindOf(b.name)) ||
                    a.name.localeCompare(b.name),
            ),
        [files],
    );
    const [selected, setSelected] = useState<string | null>(null);
    const [html, setHtml] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);

    // Default to the first CPL when the file set changes — it's the
    // most interesting document in a package.
    useEffect(() => {
        const first = sorted.find((f) => kindOf(f.name) === 'CPL') ?? sorted[0];
        setSelected(first?.name ?? null);
    }, [sorted]);

    const current = sorted.find((f) => f.name === selected) ?? null;

    useEffect(() => {
        if (!current) return;
        let cancelled = false;
        setHtml(null);
        setError(null);
        highlightXml(current.text)
            .then((out) => !cancelled && setHtml(out))
            .catch((e) => !cancelled && setError(String(e)));
        return () => {
            cancelled = true;
        };
    }, [current]);

    if (sorted.length === 0) return null;

    const copy = () => {
        if (!current) return;
        navigator.clipboard?.writeText(current.text).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1200);
        });
    };

    return (
        <div className="pfb mt-6">
            <div className="flex items-center justify-between mb-2.5">
                <h2 className="text-sm font-semibold flex items-center gap-1.5">
                    <span className="text-amber-600"><FileCodeIcon /></span> Package Files{' '}
                    <span className="font-normal text-zinc-400 dark:text-zinc-500">({sorted.length})</span>
                </h2>
            </div>
            <div className="border border-zinc-200 dark:border-zinc-700 rounded-xl overflow-hidden bg-white dark:bg-zinc-900 grid grid-cols-1 md:grid-cols-[260px_minmax(0,1fr)]">
                <ul
                    className="m-0 p-1.5 list-none flex flex-col gap-0.5 border-b md:border-b-0 md:border-r border-zinc-200 dark:border-zinc-700 max-h-48 md:max-h-[520px] overflow-y-auto"
                    role="listbox"
                    aria-label="Package files"
                >
                    {sorted.map((f) => {
                        const kind = kindOf(f.name);
                        const active = f.name === selected;
                        return (
                            <li key={f.name} className="m-0">
                                <button
                                    type="button"
                                    role="option"
                                    aria-selected={active}
                                    onClick={() => setSelected(f.name)}
                                    title={f.name}
                                    className={`w-full grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2 px-2 py-1.5 rounded-md text-left text-xs transition-colors cursor-pointer ${
                                        active
                                            ? 'bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100'
                                            : 'text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50'
                                    }`}
                                >
                                    <span className={`inline-flex justify-center px-1.5 py-px rounded-md text-[10px] font-semibold tracking-wide ${KIND_CLASS[kind]}`}>
                                        {kind}
                                    </span>
                                    <span className="font-mono text-[11px] truncate">{f.name}</span>
                                    <span className="text-[10px] font-mono text-zinc-400 dark:text-zinc-500">
                                        {formatSize(new Blob([f.text]).size)}
                                    </span>
                                </button>
                            </li>
                        );
                    })}
                </ul>
                <div className="flex flex-col min-w-0">
                    {current && (
                        <div className="flex items-center justify-between gap-2 px-3 py-2 border-b border-zinc-200 dark:border-zinc-700">
                            <span className="text-[11px] font-mono text-zinc-400 dark:text-zinc-500 truncate">{current.name}</span>
                            <button
                                type="button"
                                onClick={copy}
                                className="px-2.5 py-1 text-[11px] font-medium border border-zinc-200 dark:border-zinc-700 rounded-md bg-white dark:bg-zinc-900 text-zinc-400 dark:text-zinc-500 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 hover:text-amber-600 cursor-pointer transition-colors"
                            >
                                {copied ? 'Copied' : 'Copy'}
                            </button>
                        </div>
                    )}
                    <div className="flex-1 overflow-auto max-h-[520px] bg-zinc-50/60 dark:bg-zinc-950/40">
                        {error ? (
                            <pre className="m-0 px-4 py-3 text-[11px] leading-relaxed font-mono">{current?.text}</pre>
                        ) : html ? (
                            // Shiki HTML-escapes every token of the source, so
                            // user-dropped XML can't inject markup here.
                            <div dangerouslySetInnerHTML={{ __html: html }} />
                        ) : (
                            <div className="px-4 py-3 text-xs text-zinc-400 dark:text-zinc-500">Highlighting…</div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
