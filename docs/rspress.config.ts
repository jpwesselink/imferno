import path from "node:path";
import { defineConfig } from "@rspress/core";

const docsVersion = process.env.DOCS_VERSION;
const base = docsVersion ? `/imferno/${docsVersion}/` : "/imferno/";

export default defineConfig({
  root: "docs",
  base,
  route: {
    exclude: ["components/**"],
  },
  title: "imferno",
  description: "SMPTE ST-2067 Interoperable Master Format for Rust, Node.js, and WebAssembly",
  globalStyles: path.join(__dirname, "docs/styles/index.css"),
  head: [
    ['script', { src: `${base}version-switcher.js`, defer: '' }],
  ],
  themeConfig: {
    socialLinks: [
      { icon: "github", mode: "link", content: "https://github.com/jpwesselink/imferno" },
    ],
    nav: [
      { text: "Try in browser", link: "/#playground" },
      { text: "Guide", link: "/guide/" },
      { text: "Reference", link: "/reference/rust" },
    ],
    search: {
      mode: "local",
    },
    sidebar: {
      "/guide/": [
        { text: "Introduction", link: "/guide/" },
        { text: "Getting Started", link: "/guide/quick-start" },
        { text: "Validation", link: "/guide/validation" },
        { text: "Configuration", link: "/guide/config" },
        { text: "Validation Codes", link: "/guide/codes" },
        { text: "IMF Packages", link: "/guide/packages" },
        { text: "Examples", link: "/guide/examples" },
      ],
      "/reference/": [
        { text: "Rust", link: "/reference/rust" },
        { text: "WASM", link: "/reference/wasm" },
        { text: "Node.js", link: "/reference/node" },
        { text: "CLI", link: "/reference/cli" },
        { text: "Standards", link: "/reference/standards" },
      ],
    },
  },
});
