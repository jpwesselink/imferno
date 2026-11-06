import { useEffect, useMemo, useState } from 'react';
import { base } from './base';

// Rule severity controls for the playground. Styled with the package
// viewer's Tailwind vocabulary (zinc cards, segmented tabs, amber accents).

export type RuleSeverity = 'off' | 'info' | 'warn' | 'error' | 'critical';

// Mildest first. Colours match the viewer's issue list.
export const SEVERITY_OPTIONS: Array<{ value: RuleSeverity; label: string; activeClass: string }> = [
    { value: 'off', label: 'Off', activeClass: 'text-zinc-500 dark:text-zinc-400' },
    { value: 'info', label: 'Info', activeClass: 'text-blue-500' },
    { value: 'warn', label: 'Warning', activeClass: 'text-amber-600' },
    { value: 'error', label: 'Error', activeClass: 'text-red-600' },
    { value: 'critical', label: 'Critical', activeClass: 'text-red-700 dark:text-red-500' },
];

export function SeverityControl({
    value,
    defaultValue,
    onChange,
    label,
    compact = false,
}: {
    value: RuleSeverity;
    defaultValue?: RuleSeverity;
    onChange: (v: RuleSeverity) => void;
    label: string;
    compact?: boolean;
}) {
    return (
        <div
            className={`flex flex-shrink-0 bg-zinc-100 dark:bg-zinc-800 rounded-lg ${compact ? 'gap-0.5 p-0.5' : 'gap-1 p-1'}`}
            role="radiogroup"
            aria-label={label}
        >
            {SEVERITY_OPTIONS.map((opt) => {
                const active = value === opt.value;
                const isDefault = defaultValue === opt.value;
                return (
                    <button
                        key={opt.value}
                        type="button"
                        role="radio"
                        aria-checked={active}
                        title={isDefault ? `${opt.label} (default)` : opt.label}
                        onClick={() => onChange(opt.value)}
                        className={`relative rounded-md font-medium transition-colors cursor-pointer ${
                            compact ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-[11px]'
                        } ${
                            active
                                ? `bg-white dark:bg-zinc-900 shadow-sm ${opt.activeClass}`
                                : 'text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200'
                        }`}
                    >
                        {opt.label}
                        {isDefault && (
                            <span className="absolute top-0.5 right-0.5 w-1 h-1 rounded-full bg-zinc-400 dark:bg-zinc-500" aria-hidden />
                        )}
                    </button>
                );
            })}
        </div>
    );
}

// ─── advanced: full catalogue from docs/public/rules.json ───────────────────

interface CatalogueRule {
    code: string;
    description: string;
    defaultSeverity: RuleSeverity;
    category: string;
}
interface CatalogueGroup {
    title: string | null;
    rules: CatalogueRule[];
}
interface CatalogueSection {
    title: string;
    groups: CatalogueGroup[];
}

const Chevron = ({ open }: { open: boolean }) => (
    <span className={`flex transition-transform duration-200 ${open ? '' : '-rotate-90'}`}>
        <svg width={12} height={12} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
            <path d="m6 9 6 6 6-6" />
        </svg>
    </span>
);

function matches(rule: CatalogueRule, q: string): boolean {
    return rule.code.toLowerCase().includes(q) || rule.description.toLowerCase().includes(q);
}

export function AdvancedRules({
    config,
    setRule,
    resetAll,
    isPristine,
}: {
    config: Record<string, RuleSeverity>;
    /** `null` clears the override (back to the catalogue default). */
    setRule: (code: string, severity: RuleSeverity | null) => void;
    /** Restore the playground's default rule config. */
    resetAll: () => void;
    /** True when the config equals the playground defaults. */
    isPristine: boolean;
}) {
    const [sections, setSections] = useState<CatalogueSection[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [query, setQuery] = useState('');
    const [open, setOpen] = useState<Set<string>>(new Set());

    useEffect(() => {
        let cancelled = false;
        fetch(`${base}rules.json`)
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json();
            })
            .then((d) => !cancelled && setSections(d.sections))
            .catch((e) => !cancelled && setError(String(e)));
        return () => {
            cancelled = true;
        };
    }, []);

    const q = query.trim().toLowerCase();
    const filtered = useMemo(() => {
        if (!sections) return [];
        if (!q) return sections;
        return sections
            .map((s) => ({
                ...s,
                groups: s.groups
                    .map((g) => ({ ...g, rules: g.rules.filter((r) => matches(r, q)) }))
                    .filter((g) => g.rules.length > 0),
            }))
            .filter((s) => s.groups.length > 0);
    }, [sections, q]);

    const total = useMemo(
        () => new Set((sections ?? []).flatMap((s) => s.groups.flatMap((g) => g.rules.map((r) => r.code)))).size,
        [sections],
    );
    // Overrides relative to the catalogue defaults.
    const overrideCount = useMemo(() => {
        const defaults = new Map<string, RuleSeverity>();
        for (const s of sections ?? []) for (const g of s.groups) for (const r of g.rules) defaults.set(r.code, r.defaultSeverity);
        return Object.entries(config).filter(([code, sev]) => defaults.get(code) !== sev).length;
    }, [sections, config]);

    if (error) {
        return <p className="text-xs text-red-600 px-4 py-3 m-0">Could not load the rule catalogue: {error}</p>;
    }
    if (!sections) {
        return <p className="text-xs text-zinc-400 dark:text-zinc-500 px-4 py-3 m-0">Loading rule catalogue…</p>;
    }

    const toggle = (title: string) =>
        setOpen((prev) => {
            const next = new Set(prev);
            next.has(title) ? next.delete(title) : next.add(title);
            return next;
        });

    return (
        <div className="px-4 py-3 flex flex-col gap-3">
            <div className="flex items-center gap-2 flex-wrap">
                <input
                    type="search"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder={`Filter ${total} rules by code or description…`}
                    className="flex-1 min-w-[200px] px-3 py-1.5 text-xs rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 outline-none focus:border-amber-600/60 focus:ring-1 focus:ring-amber-600/20"
                />
                <span className="text-[11px] text-zinc-400 dark:text-zinc-500">
                    {overrideCount} override{overrideCount === 1 ? '' : 's'}
                </span>
                <button
                    type="button"
                    onClick={resetAll}
                    disabled={isPristine}
                    className="px-2.5 py-1 text-[11px] font-medium border border-zinc-200 dark:border-zinc-700 rounded-md bg-white dark:bg-zinc-900 text-zinc-400 dark:text-zinc-500 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 hover:text-amber-600 cursor-pointer disabled:cursor-default disabled:opacity-50 disabled:hover:text-zinc-400 transition-colors"
                >
                    Reset to defaults
                </button>
            </div>

            {filtered.length === 0 && (
                <p className="text-xs text-zinc-400 dark:text-zinc-500 text-center py-6 m-0">No rules match “{query}”.</p>
            )}

            <div className="flex flex-col gap-2">
                {filtered.map((sec) => {
                    const isOpen = q.length > 0 || open.has(sec.title);
                    const ruleCount = sec.groups.reduce((n, g) => n + g.rules.length, 0);
                    const overridden = sec.groups.reduce(
                        (n, g) => n + g.rules.filter((r) => config[r.code] !== undefined).length,
                        0,
                    );
                    return (
                        <div
                            key={sec.title}
                            className={`border rounded-xl overflow-hidden bg-white dark:bg-zinc-900 ${
                                isOpen ? 'border-zinc-300 dark:border-zinc-600' : 'border-zinc-200 dark:border-zinc-700'
                            }`}
                        >
                            <button
                                type="button"
                                onClick={() => toggle(sec.title)}
                                className="w-full flex items-center gap-2 px-3.5 py-2.5 text-left hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors cursor-pointer"
                            >
                                <span className={isOpen ? 'text-amber-600' : 'text-zinc-400 dark:text-zinc-500'}>
                                    <Chevron open={isOpen} />
                                </span>
                                <span className="flex-1 text-xs font-semibold text-zinc-900 dark:text-zinc-100">{sec.title}</span>
                                {overridden > 0 && (
                                    <span className="px-1.5 rounded-md text-[10px] font-medium leading-[18px] bg-yellow-500/10 text-yellow-600 border border-yellow-500/20">
                                        {overridden} changed
                                    </span>
                                )}
                                <span className="text-[11px] text-zinc-400 dark:text-zinc-500">{ruleCount}</span>
                            </button>
                            {isOpen && (
                                <div className="border-t border-zinc-200 dark:border-zinc-700">
                                    {sec.groups.map((g, gi) => (
                                        <div key={`${sec.title}-${g.title ?? gi}`}>
                                            {g.title && (
                                                <div className="px-3.5 pt-3 pb-1 text-[10px] font-semibold uppercase tracking-wider text-zinc-400 dark:text-zinc-500">
                                                    {g.title}
                                                </div>
                                            )}
                                            <div className="divide-y divide-zinc-100 dark:divide-zinc-800">
                                                {g.rules.map((r) => {
                                                    const override = config[r.code];
                                                    const current = override ?? r.defaultSeverity;
                                                    const changed = override !== undefined && override !== r.defaultSeverity;
                                                    return (
                                                        <div
                                                            key={`${g.title ?? gi}-${r.code}`}
                                                            className={`flex items-center gap-3 flex-wrap px-3.5 py-2 ${
                                                                changed ? 'shadow-[inset_2px_0_0_var(--color-amber-600)]' : ''
                                                            }`}
                                                        >
                                                            <div className="flex-1 min-w-[220px]">
                                                                <code className="block text-[11px] font-mono text-zinc-700 dark:text-zinc-300 break-all">
                                                                    {r.code}
                                                                </code>
                                                                <span className="block text-[11px] text-zinc-400 dark:text-zinc-500 mt-0.5">
                                                                    {r.description}
                                                                    {changed && (
                                                                        <button
                                                                            type="button"
                                                                            onClick={() => setRule(r.code, null)}
                                                                            className="ml-2 text-amber-600 hover:underline cursor-pointer"
                                                                        >
                                                                            reset
                                                                        </button>
                                                                    )}
                                                                </span>
                                                            </div>
                                                            <SeverityControl
                                                                compact
                                                                label={`Severity for ${r.code}`}
                                                                value={current}
                                                                defaultValue={r.defaultSeverity}
                                                                onChange={(v) => setRule(r.code, v === r.defaultSeverity ? null : v)}
                                                            />
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
