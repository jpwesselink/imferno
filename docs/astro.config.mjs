// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import react from '@astrojs/react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    site: 'https://jpwesselink.github.io/imferno',
    base: '/imferno',
    integrations: [
        starlight({
            title: 'imferno',
            description: 'SMPTE ST 2067 Interoperable Master Format for Rust',
            customCss: ['./src/styles/custom.css'],
            social: [
                { icon: 'github', label: 'GitHub', href: 'https://github.com/jpwesselink/imferno' },
            ],
            sidebar: [],
        }),
        react(),
    ],
    vite: {
        plugins: [tailwindcss()],
    },
});
