// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import react from '@astrojs/react';
import tailwindcss from '@tailwindcss/vite';

// When DOCS_VERSION is set (e.g. "v1.0.0"), docs are built for /imferno/v1.0.0/
const docsVersion = process.env.DOCS_VERSION;
const base = docsVersion ? `/imferno/${docsVersion}` : '/imferno';

export default defineConfig({
    site: 'https://jpwesselink.github.io/imferno',
    base,
    integrations: [
        starlight({
            title: 'imferno',
            description: 'SMPTE ST 2067 Interoperable Master Format for Rust',
            customCss: ['./src/styles/custom.css'],
            social: [
                { icon: 'github', label: 'GitHub', href: 'https://github.com/jpwesselink/imferno' },
            ],
            tableOfContents: false,
            sidebar: [
                { label: 'Introduction', slug: 'guide/introduction' },
                { label: 'Getting Started', slug: 'guide/quick-start' },
                { label: 'Validation', slug: 'guide/validation' },
                { label: 'Configuration', slug: 'guide/config' },
                {
                    label: 'API Reference',
                    items: [
                        { label: 'Rust', slug: 'reference/rust' },
                        { label: 'WASM', slug: 'reference/wasm' },
                        { label: 'Node.js', slug: 'reference/node' },
                        { label: 'CLI', slug: 'reference/cli' },
                    ],
                },
                {
                    label: 'Validation Codes',
                    items: [
                        { label: 'ST 429-9', slug: 'reference/codes/st429-9' },
                        { label: 'ST 377-1', slug: 'reference/codes/st377-1' },
                        { label: 'ST 2067-2', slug: 'reference/codes/st2067-2' },
                        { label: 'ST 2067-3', slug: 'reference/codes/st2067-3' },
                        { label: 'ST 2067-9', slug: 'reference/codes/st2067-9' },
                        { label: 'ST 2067-21', slug: 'reference/codes/st2067-21' },
                        { label: 'ST 2067-201', slug: 'reference/codes/st2067-201' },
                        { label: 'ST 2067-202', slug: 'reference/codes/st2067-202' },
                        { label: 'imferno', slug: 'reference/codes/imferno' },
                    ],
                },
            ],
            components: {
                SiteTitle: './src/components/SiteTitle.astro',
            },
        }),
        react(),
    ],
    vite: {
        plugins: [tailwindcss()],
        optimizeDeps: {
            include: ['react', 'react-dom', 'react/jsx-runtime', 'react/jsx-dev-runtime'],
        },
    },
});
