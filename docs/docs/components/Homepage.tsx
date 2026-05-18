import { useEffect } from 'react';
import ImfPlayground from './ImfPlayground';
import './Homepage.css';

declare global {
  interface Window {
    __imfWasm?: any;
    __imfWasmError?: string;
  }
}

// Use Vite's base URL (set by Rspress from rspress.config.ts base), ensure trailing slash
const rawBase = typeof import.meta !== 'undefined' ? (import.meta as any).env?.BASE_URL || '/imferno/' : '/imferno/';
const base = rawBase.endsWith('/') ? rawBase : rawBase + '/';

export default function Homepage() {
  // Load WASM module on mount (browser only — skip during SSR)
  useEffect(() => {
    if (typeof window === 'undefined' || typeof document === 'undefined') return;
    if (window.__imfWasm) return;
    const script = document.createElement('script');
    script.type = 'module';
    script.textContent = `
      try {
        const mod = await import('${base}wasm/imferno_wasm.js?v=${Date.now()}');
        await mod.default('${base}wasm/imferno_wasm_bg.wasm?v=${Date.now()}');
        window.__imfWasm = mod;
      } catch (e) {
        window.__imfWasmError = String(e);
      }
      window.dispatchEvent(new CustomEvent('imf-wasm-ready'));
    `;
    document.head.appendChild(script);
  }, []);

  return (
    <div className="homepage">
      {/* Hero */}
      <section className="hero-section">
        <h1 className="hero-title">imferno</h1>
        <p className="hero-subtitle"><a href="https://www.smpte.org/standards/st2067" className="hero-link">SMPTE ST-2067</a><br />for Rust and JavaScript.</p>
        <p className="hero-desc">
          Parse, validate, and inspect IMF packages with native Node.js bindings
          and a WebAssembly target for the browser.
        </p>
      </section>

      {/* Feature cards */}
      <section className="section">
        <div className="card-grid four">
          <a className="feat-card" href={`${base}reference/standards`}>
            <h3>IMF core + applications</h3>
            <p>Parse AssetMaps, PKLs, and CPLs per SMPTE ST 2067-2, ST 2067-3, and Application profiles including ST 2067-21 and ST 2067-201.</p>
          </a>
          <a className="feat-card" href={`${base}guide/config`}>
            <h3>Flexible strictness</h3>
            <p>250+ validation rules across 8 SMPTE standards. Tune each rule individually: set any code to <code className="icode">off</code>, <code className="icode">info</code>, <code className="icode">warn</code>, <code className="icode">error</code>, or <code className="icode">critical</code> to match your workflow.</p>
          </a>
          <a className="feat-card" href={`${base}guide/quick-start`}>
            <h3>Rust, Node.js &amp; WASM</h3>
            <p>Use the native Rust library or CLI directly, <code className="icode">@imferno/node</code> bindings for server-side pipelines, or the WebAssembly build for the browser.</p>
          </a>
          <a className="feat-card" href={`${base}guide/validation`}>
            <h3>IMF reference corpus</h3>
            <p>Tested against the Netflix Photon MERIDIAN reference corpus. ST 2067-2, ST 2067-3, ST 2067-21, and ST 2067-201 fully covered.</p>
          </a>
        </div>
      </section>

      {/* Get started */}
      <section className="section">
        <div className="card-grid two">
          <a className="get-started-card" href={`${base}reference/cli`}>
            <div className="gs-header"><span className="gs-icon">$</span><span className="gs-label">npx</span></div>
            <p className="gs-desc">No global install needed. Run the latest version directly.</p>
            <div className="codeblock">
              <code>npx imferno@latest validate ./my-package</code>
            </div>
          </a>
          <a className="get-started-card" href={`${base}reference/rust`}>
            <div className="gs-header"><span className="gs-icon">🦀</span><span className="gs-label">Rust</span></div>
            <p className="gs-desc">Install the native Rust binary. Offline, no runtime.</p>
            <div className="codeblock">
              <code>cargo install imferno</code>
              <code className="dim">imferno validate ./my-package</code>
            </div>
          </a>
          <a className="get-started-card" href={`${base}reference/node`}>
            <div className="gs-header"><span className="gs-icon">📦</span><span className="gs-label">Node.js</span></div>
            <p className="gs-desc">Native bindings for server-side pipelines and automation.</p>
            <div className="codeblock">
              <code>npm install @imferno/node</code>
            </div>
          </a>
          <a className="get-started-card" href={`${base}reference/wasm`}>
            <div className="gs-header"><span className="gs-icon">🌐</span><span className="gs-label">WASM</span></div>
            <p className="gs-desc">ESM module powered by WebAssembly. Use it in any browser or bundler.</p>
            <div className="codeblock">
              <code>npm install @imferno/wasm</code>
            </div>
          </a>
        </div>
      </section>

      {/* IMF Playground */}
      <section className="section">
        <ImfPlayground />
      </section>

      {/* Standards coverage */}
      <section className="section">
        <h2 className="section-title">Standards coverage</h2>
        <p className="section-subtitle">SMPTE ST-2067 implementation status.</p>
        <div className="table-wrap">
          <table className="std-table">
            <thead>
              <tr><th>Standard</th><th>Description</th><th>Status</th></tr>
            </thead>
            <tbody>
              <tr><td colSpan={3} className="section-row">Complete</td></tr>
              <tr><td>ST 429-9</td><td>Volume Index / Asset Map</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td>ST 2067-2:2013, :2016, :2020</td><td>Core Constraints &amp; Packing List</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td>ST 2067-3:2013, :2016, :2020</td><td>Composition Playlist</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td>ST 2067-21:2020, :2023, :2025</td><td>Application #2E (UHD/HDR)</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td>ST 2067-201:2019, :2021</td><td>IAB Level 0 Plug-in</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td>ST 2067-9:2018</td><td>Sidecar Composition Map</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td>ST 2067-202:2022</td><td>ISXD Plug-in</td><td><span className="badge done">Complete</span></td></tr>
              <tr><td colSpan={3} className="section-row">Partial</td></tr>
              <tr><td>ST 377-1:2011</td><td>MXF File Format</td><td><span className="badge partial">Partial</span></td></tr>
              <tr><td colSpan={3} className="section-row">Not implemented</td></tr>
              <tr><td>ST 429-8</td><td>D-Cinema Packing List</td><td><span className="badge none">Not implemented</span></td></tr>
              <tr><td>ST 2067-100:2014</td><td>Output Profile List</td><td><span className="badge none">Not implemented</span></td></tr>
              <tr><td>ST 2067-203:2023</td><td>S-ADM Audio Plug-in</td><td><span className="badge none">Not implemented</span></td></tr>
              <tr><td>ST 377-41</td><td>MXF MGA / S-ADM Virtual Tracks</td><td><span className="badge none">Not implemented</span></td></tr>
              <tr><td>ST 379-2:2010</td><td>MXF Generic Container</td><td><span className="badge none">Not implemented</span></td></tr>
              <tr><td>ST 422:2014</td><td>JPEG 2000 in MXF</td><td><span className="badge none">Not implemented</span></td></tr>
            </tbody>
          </table>
        </div>
      </section>

      {/* Sponsor */}
      <section className="section" style={{ textAlign: 'center', paddingBottom: '4rem' }}>
        <p className="sponsor-text">
          If imferno is useful to your workflow, consider{' '}
          <a href="https://github.com/sponsors/jpwesselink" className="sponsor-link">sponsoring the project</a>.
        </p>
      </section>
    </div>
  );
}
