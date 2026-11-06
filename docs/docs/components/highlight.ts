// Lazy XML syntax highlighter for the playground file browser.
//
// Uses Shiki (same engine Rspress uses for code blocks) in its fine-grained
// form: core + the JavaScript regex engine (no oniguruma WASM) + only the
// XML grammar and two themes. Loaded on first use so the homepage doesn't
// pay for it until someone opens a file.
//
// Output uses Shiki's dual-theme CSS variables; `Homepage.css` switches
// between them on `html.rp-dark`.

import type { HighlighterCore } from 'shiki/core';

let highlighter: Promise<HighlighterCore> | null = null;

function getHighlighter(): Promise<HighlighterCore> {
    if (!highlighter) {
        highlighter = (async () => {
            const [{ createHighlighterCore }, { createJavaScriptRegexEngine }] = await Promise.all([
                import('shiki/core'),
                import('shiki/engine/javascript'),
            ]);
            return createHighlighterCore({
                engine: createJavaScriptRegexEngine(),
                langs: [import('@shikijs/langs/xml')],
                themes: [import('@shikijs/themes/github-light'), import('@shikijs/themes/github-dark')],
            });
        })();
        // Allow a retry if loading failed (e.g. a flaky network chunk).
        highlighter.catch(() => {
            highlighter = null;
        });
    }
    return highlighter;
}

/** Highlight XML source to an HTML string (a `<pre class="shiki">` block). */
export async function highlightXml(source: string): Promise<string> {
    const hl = await getHighlighter();
    return hl.codeToHtml(source, {
        lang: 'xml',
        themes: { light: 'github-light', dark: 'github-dark' },
        defaultColor: false,
    });
}
