# imferno docs

Documentation site for the `imferno` workspace, built with [Astro Starlight](https://starlight.astro.build).

## Development

```bash
pnpm install
pnpm dev        # dev server at localhost:4321
pnpm build      # production build to ./dist/
pnpm preview    # preview production build locally
```

## Structure

```
docs/
├── src/
│   ├── assets/
│   └── content/
│       └── docs/   # .md / .mdx pages (one file = one route)
├── astro.config.mjs
└── package.json
```

Add pages by creating `.md` or `.mdx` files under `src/content/docs/`. The filename becomes the URL path.

## Related

- Workspace root: [../README.md](../README.md)
- WASM bindings: [../crates/imferno-wasm/README.md](../crates/imferno-wasm/README.md)
