import { useEffect, useMemo, useState } from 'react';
import { highlightXml } from './highlight';

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

    return (
        <div className="pfb">
            <div className="pfb-header">Package files</div>
            <div className="pfb-body">
                <ul className="pfb-list" role="listbox" aria-label="Package files">
                    {sorted.map((f) => {
                        const kind = kindOf(f.name);
                        const active = f.name === selected;
                        return (
                            <li key={f.name}>
                                <button
                                    type="button"
                                    role="option"
                                    aria-selected={active}
                                    className={`pfb-item${active ? ' is-active' : ''}`}
                                    onClick={() => setSelected(f.name)}
                                    title={f.name}
                                >
                                    <span className={`pfb-kind pfb-kind-${kind.toLowerCase()}`}>{kind}</span>
                                    <span className="pfb-name">{f.name}</span>
                                    <span className="pfb-size">{formatSize(new Blob([f.text]).size)}</span>
                                </button>
                            </li>
                        );
                    })}
                </ul>
                <div className="pfb-viewer">
                    {current && (
                        <div className="pfb-viewer-bar">
                            <span className="pfb-viewer-name">{current.name}</span>
                            <button
                                type="button"
                                className="pfb-copy"
                                onClick={() => navigator.clipboard?.writeText(current.text)}
                            >
                                Copy
                            </button>
                        </div>
                    )}
                    <div className="pfb-code">
                        {error ? (
                            <pre className="pfb-plain">{current?.text}</pre>
                        ) : html ? (
                            // Shiki HTML-escapes every token of the source, so
                            // user-dropped XML can't inject markup here.
                            <div dangerouslySetInnerHTML={{ __html: html }} />
                        ) : (
                            <div className="pfb-loading">Highlighting…</div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
