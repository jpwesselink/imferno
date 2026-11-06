> For AI agents: the complete documentation index is available at /imferno/pr-97/llms.txt, the full documentation bundle is available at /imferno/pr-97/llms-full.txt.

# imferno

[SMPTE ST-2067](https://www.smpte.org/standards/st2067)
for Rust and JavaScript.

Parse, validate, and inspect IMF packages with native Node.js bindings and a WebAssembly target for the browser.

[### IMF core + applications

Parse AssetMaps, PKLs, CPLs, and Sidecar Composition Maps across the IMF core (ST 2067-2/-3/-9, ST 429-9) plus application profiles: App #2E (ST 2067-21), IAB Level 0 through the 2026 Annex E revision (ST 2067-201), ISXD (ST 2067-202), and S-ADM/ADM audio essence detection (ST 2067-203/-204). MXF header metadata via RegXML (ST 377-1/-4).

](/imferno/pr-97/reference/standards)[### Flexible strictness

180+ typed validation rules across 10 SMPTE standards, plus XSD pre-pass and imferno-native structural checks. Tune each rule individually: set any code to `off`, `info`, `warn`, `error`, or `critical` to match your workflow.

](/imferno/pr-97/guide/config)[### Rust, Node.js & WASM

Use the native Rust library or CLI directly, `@imferno/node` bindings for server-side pipelines, or the WebAssembly build for the browser.

](/imferno/pr-97/guide/quick-start)[### IMF reference corpus

Tested against the Netflix Photon MERIDIAN corpus, the Fraunhofer SMPTE working-group ST 2067-203/-204 corpus, and vendored fixtures for App #2E, App5 (IMAX HDR), IAB, ISXD, HT (JPEG 2000 High Throughput), S-ADM, and Netflix–Sony Plugfest.

](/imferno/pr-97/guide/validation)

[$npxNo global install needed. Run the latest version directly.

`npx imferno@latest validate ./my-package`](/imferno/pr-97/reference/cli)[🦀RustInstall the native Rust binary. Offline, no runtime.

`cargo install imferno``imferno validate ./my-package`](/imferno/pr-97/reference/rust)[📦Node.jsNative bindings for server-side pipelines and automation.

`npm install @imferno/node`](/imferno/pr-97/reference/node)[🌐WASMESM module powered by WebAssembly. Use it in any browser or bundler.

`npm install @imferno/wasm`](/imferno/pr-97/reference/wasm)

## Try it in your browser

Drop any IMF XML file — the parser runs entirely in WebAssembly. Nothing leaves your browser.

Loading WebAssembly module…

## Standards coverage

SMPTE ST-2067 implementation status.

| Standard | Description | Status |
| --- | --- | --- |
| Complete |
| ST 429-9 | Volume Index / Asset Map | Complete |
| ST 2067-2:2013, :2016, :2020 | Core Constraints & Packing List | Complete |
| ST 2067-3:2013, :2016, :2020 | Composition Playlist | Complete |
| ST 2067-21:2020, :2023, :2025 | Application #2E (UHD/HDR) | Complete |
| ST 2067-201:2019, :2021, :2026 | IAB Level 0 Plug-in (incl. 2026 Annex E IABChannelSubDescriptor recommendation) | Complete |
| ST 2067-9:2018 | Sidecar Composition Map | Complete |
| ST 2067-202:2022 | ISXD Plug-in | Complete |
| ST 377-1:2011 | MXF File Format (KLV, partition packs, header metadata via RegXML — essence sample decoding is the codec spec's job, see below) | Complete |
| ST 377-4:2012 | MXF MCA (Multi-Channel Audio) Labeling | Complete |
| Partial |
| ST 2067-203:2023 | S-ADM Audio Plug-in (essence-layer MGA/S-ADM detection; CPL-level plug-in semantics pending) | Partial |
| ST 2067-204:2023 | ADM Audio Plug-in (essence-layer ST 2131 ADM detection; CPL-level plug-in semantics pending) | Partial |
| Not implemented |
| ST 429-8 | D-Cinema Packing List | Not implemented |
| ST 2067-100:2014 | Output Profile List | Not implemented |
| ST 377-41 | MXF MGA / S-ADM Virtual Tracks | Not implemented |
| ST 379-2:2010 | MXF Generic Container | Not implemented |
| ST 422:2014 | JPEG 2000 in MXF | Not implemented |

If imferno is useful to your workflow, consider [sponsoring the project](https://github.com/sponsors/jpwesselink).

